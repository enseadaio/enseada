use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde::export::Formatter;
use uuid::Uuid;

use crate::couchdb::db::Database;
use crate::couchdb::guid::Guid;
use crate::error::Error;
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
        let model = &mut self.model;
        let mut principals = HashMap::new();
        let mut roles = HashMap::new();

        let rules = self.db.list_all::<Rule>("rule").await?;
        for row in rules.rows {
            let rule = &row.doc;
            let permission = Permission::new(&rule.obj, &rule.act);
            let role = self.db.get::<RoleAssignment>(&RoleAssignment::build_guid(&rule.sub).to_string()).await?;
            if let Some(role) = role {
                let mut role = Role::new(role.role);
                role.add_permission(permission);
                roles.insert(role.name().to_string(), role);
            } else {
                let mut principal = Principal::new(rule.sub.clone());
                principal.add_permission(permission);
                principals.insert(principal.name().to_string(), principal);
            }
        }

        let role_assignments = self.db.list_all::<RoleAssignment>("role").await?;
        for row in role_assignments.rows {
            let role_assignment = &row.doc;

            if let Some(principal) = principals.get_mut(&role_assignment.sub) {
                let role = roles.get(&role_assignment.role);
                let role = match role {
                    Some(role) => role.clone(),
                    None => Role::new(role_assignment.role.clone()),
                };
                principal.add_role(role);
            }
        }

        model.set_principals(principals);
        Ok(())
    }

    pub fn check(&self, sub: &str, obj: &str, act: &str) -> Result<(), EvaluationError> {
        match self.model.check(sub, obj, act) {
            EvaluationResult::Granted => Ok(()),
            EvaluationResult::Denied => Err(EvaluationError::Denied),
        }
    }

    pub async fn add_rule(&self, sub: &str, obj: &str, act: &str) -> Result<(), Error> {
        let rule = Rule::new(sub.to_string(), obj.to_string(), act.to_string());
        self.db.put(&rule.id.to_string(), rule).await?;

        Ok(())
    }

    pub async fn add_role_to_principal(&self, sub: &str, role: &str) -> Result<(), Error> {
        let assignment = RoleAssignment::new(sub.to_string(), role.to_string());
        self.db.put(&assignment.id.to_string(), assignment).await?;

        Ok(())
    }

    pub async fn remove_role_from_principal(&self, sub: &str, role: &str) -> Result<(), Error> {
        if let Some(assignment) = self.db.get::<RoleAssignment>(&RoleAssignment::build_guid(role).to_string()).await? {
            self.db.delete(&assignment.id.to_string(), &assignment.rev.unwrap()).await?;
        }

        Ok(())
    }

    pub async fn get_principal_roles(&self, sub: &str) -> Result<Vec<String>, Error> {
        let response = self.db.find::<RoleAssignment>(serde_json::json!({
            "sub": sub
        })).await?;

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
    sub: String,
    obj: String,
    act: String,
}

impl Rule {
    pub fn build_guid(id: &str) -> Guid {
        Guid::from(format!("rule:{}", id))
    }

    pub fn new(sub: String, obj: String, act: String) -> Self {
        let uuid = Uuid::new_v4().to_string();
        let id = Self::build_guid(&uuid);
        Rule { id, rev: None, sub, obj, act }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleAssignment {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    sub: String,
    role: String,
}

impl RoleAssignment {
    pub fn build_guid(id: &str) -> Guid {
        Guid::from(format!("role:{}", id))
    }

    pub fn new(sub: String, role: String) -> Self {
        let id = Self::build_guid(&role);
        RoleAssignment { id, rev: None, sub, role }
    }
}