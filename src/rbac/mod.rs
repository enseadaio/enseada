use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;

use http::StatusCode;
use serde::{Deserialize, Deserializer, Serialize};
use serde::export::Formatter;
use uuid::Uuid;

use crate::couchdb::db::Database;
use crate::error::Error;
use crate::guid::Guid;
use crate::pagination::{Cursor, Page};
use crate::rbac::model::{EvaluationResult, Model, Permission, Principal, Role};

mod model;

pub struct Enforcer {
    db: Database,
    model: Model,
}

impl Enforcer {
    pub fn new(db: Database) -> Self {
        Enforcer { db, model: Model::empty() }
    }

    pub async fn load_rules(&mut self) -> Result<(), Error> {
        log::info!("Loading RBAC rules from CouchDB");
        let model = &mut self.model;
        let mut principals = HashMap::new();
        let mut roles = HashMap::new();

        log::debug!("Loading rules");
        let rules = self.db.list_all::<Rule>("rule").await?;
        for row in rules.rows {
            let rule = &row.doc;
            log::debug!("Processing rule {:?}", rule);
            let permission = Permission::new(&rule.obj.to_string(), &rule.act);
            if rule.sub.partition() == Some("role".to_string()) {
                log::debug!("Rule has a role subject. Adding permission to it");
                let mut role = Role::new(rule.sub.id().to_string());
                role.add_permission(permission);
                roles.insert(role.name().to_string(), role);
            } else {
                log::debug!("Rule has a principal subject. Adding permission to it");
                let mut principal = Principal::new(rule.sub.to_string());
                principal.add_permission(permission);
                principals.insert(principal.name().to_string(), principal);
            }
        }

        log::debug!("Loading roles for principals");
        let role_assignments = self.db.list_all::<RoleAssignment>("role").await?;
        for row in role_assignments.rows {
            let assignment = &row.doc;

            log::debug!("Processing role assignment {:?}", assignment);
            let sub = assignment.subject.to_string();
            if !principals.contains_key(&sub) {
                principals.insert(sub.clone(), Principal::new(sub.clone()));
            }

            let principal = principals.get_mut(&sub).unwrap();
            log::debug!("Found assignment subject {:?}. Adding role to it", principal);
            let role = roles.get(&assignment.role);
            let role = match role {
                Some(role) => role.clone(),
                None => Role::new(assignment.role.clone()),
            };
            principal.add_role(role);
        }

        model.set_principals(principals);
        Ok(())
    }

    pub fn check(&self, sub: &Guid, obj: &Guid, act: &str) -> Result<(), EvaluationError> {
        let sub = &sub.to_string();
        let obj = &obj.to_string();
        log::info!("Evaluating permission sub: {}, obj: {}, act: {}", sub, obj, act);
        match self.model.check(sub, obj, act) {
            EvaluationResult::Granted => {
                log::info!("Access Granted");
                Ok(())
            }
            EvaluationResult::Denied => {
                log::warn!("Access Denied");
                Err(EvaluationError::Denied)
            }
        }
    }

    pub async fn add_permission(&self, sub: Guid, obj: Guid, act: &str) -> Result<(), Error> {
        let sub_name = sub.to_string();
        let rule = Rule::new(sub, obj, act.to_string());
        match self.db.put(&rule.id.to_string(), rule).await {
            Ok(_) => Ok(()),
            Err(err) => match err.status() {
                StatusCode::CONFLICT => {
                    let mut err = Error::from(format!("permission already assigned to {}", sub_name));
                    err.set_status(StatusCode::CONFLICT);
                    Err(err)
                }
                _ => Err(Error::from(err)),
            }
        }
    }

    pub async fn remove_permission(&self, sub: &Guid, obj: Guid, act: &str) -> Result<(), Error> {
        let sub_name = sub.to_string();
        log::debug!("Removing permission form sub {}", &sub_name);
        let id = Rule::build_guid(&sub_name, &obj.to_string(), &act.to_string());
        let rule = self.db.get::<Rule>(&id.to_string()).await?
            .ok_or_else(|| Error::new("permission not found", Some(StatusCode::NOT_FOUND)))?;
        log::debug!("Permission found, removing");
        self.db.delete(&rule.id.to_string(), &rule.rev.unwrap()).await?;
        Ok(())
    }

    pub async fn list_principal_permissions(&self, sub: &Guid, limit: usize, cursor: Option<&Cursor>) -> Result<Page<Rule>, Error> {
        log::debug!("Listing principal permissions for sub {} with limit: {} and cursor: {:?}", &sub, limit, &cursor);
        let response = self.db.find_partitioned::<Rule>("rule", serde_json::json!({
            "sub": sub.to_string(),
        }), limit, cursor.map(Cursor::to_string)).await?;

        if let Some(warning) = &response.warning {
            log::warn!("{}", warning);
        }

        Ok(Page::from_find_response(response, limit))
    }

    pub async fn add_role_to_principal(&self, sub: Guid, role: &str) -> Result<(), Error> {
        let sub_name = sub.to_string();
        let assignment = RoleAssignment::new(sub, role.to_string());
        match self.db.put(&assignment.id.to_string(), assignment).await {
            Ok(_) => Ok(()),
            Err(err) => match err.status() {
                StatusCode::CONFLICT => {
                    let mut err = Error::from(format!("role already assigned to {}", sub_name));
                    err.set_status(StatusCode::CONFLICT);
                    Err(err)
                }
                _ => Err(Error::from(err)),
            }
        }
    }

    pub async fn remove_role_from_principal(&self, sub: &Guid, role: &str) -> Result<(), Error> {
        if let Some(assignment) = self.db.get::<RoleAssignment>(&RoleAssignment::build_guid(&sub.to_string(), role).to_string()).await? {
            self.db.delete(&assignment.id.to_string(), &assignment.rev.unwrap()).await?;
        }

        Ok(())
    }

    pub async fn get_principal_roles(&self, sub: &Guid) -> Result<Vec<String>, Error> {
        // TODO: paginate
        let response = self.db.find_partitioned::<RoleAssignment>("role", serde_json::json!({
            "subject": sub.to_string()
        }), 25, None).await?;

        if let Some(warning) = response.warning {
            log::warn!("{}", warning);
        }

        Ok(response.docs.iter().map(|role| role.role.clone()).collect())
    }
}

#[derive(Debug)]
pub enum EvaluationError {
    Denied,
    Other(String),
}

impl Display for EvaluationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Denied => write!(f, "Access denied"),
            Self::Other(ms) => ms.fmt(f),
        }
    }
}

impl std::error::Error for EvaluationError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    sub: Guid,
    obj: Guid,
    act: String,
}

impl Rule {
    fn build_guid(sub: &str, obj: &str, act: &str) -> Guid {
        Guid::from(format!("rule:{}-{}-{}", sub, obj, act))
    }

    fn new(sub: Guid, obj: Guid, act: String) -> Self {
        let id = Self::build_guid(&sub.to_string(), &obj.to_string(), &act.to_string());
        Rule { id, rev: None, sub, obj, act }
    }

    pub fn subject(&self) -> &Guid {
        &self.sub
    }

    pub fn object(&self) -> &Guid {
        &self.obj
    }

    pub fn action(&self) -> &str {
        &self.act
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleAssignment {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    subject: Guid,
    role: String,
}

impl RoleAssignment {
    pub fn build_guid(sub: &str, role: &str) -> Guid {
        Guid::from(format!("role:{}-{}", role, sub))
    }

    pub fn new(subject: Guid, role: String) -> Self {
        let id = Self::build_guid(&subject.to_string(), &role);
        RoleAssignment { id, rev: None, subject, role }
    }
}
