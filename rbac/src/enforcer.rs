use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::sync::Arc;

use http::StatusCode;

use enseada::couchdb::db::Database;
use enseada::couchdb::repository::Entity;
use enseada::error::Error;
use enseada::guid::Guid;
use enseada::pagination::Page;

use crate::model::{EvaluationResult, Model, Permission, Principal, Role};
use crate::rule::{RoleAssignment, Rule};
use crate::ROOT_USER;

#[derive(Debug)]
pub struct Enforcer {
    db: Arc<Database>,
    model: Model,
}

impl Enforcer {
    pub fn new(db: Arc<Database>) -> Self {
        Enforcer {
            db,
            model: Model::empty(),
        }
    }

    #[tracing::instrument]
    pub async fn load_rules(&mut self) -> Result<(), Error> {
        log::info!("Loading RBAC rules from CouchDB");
        let model = &mut self.model;
        let mut principals = HashMap::new();
        let mut roles = HashMap::new();

        log::debug!("Loading rules");
        let rules = self.db.list_all_partitioned::<Rule>("rule").await?;
        for row in rules.rows {
            let rule = &row.doc.unwrap();
            log::debug!("Processing rule {:?}", rule);
            let permission = Permission::new(&rule.object().to_string(), &rule.action());
            let sub = rule.subject().id().to_string();
            if rule.subject().partition() == Some("role") {
                log::debug!("Rule has a role subject. Adding permission to it");
                if !roles.contains_key(&sub) {
                    roles.insert(sub.clone(), Role::new(sub.clone()));
                }
                let role = roles.get_mut(&sub).unwrap();
                role.add_permission(permission);
            } else {
                log::debug!("Rule has a principal subject. Adding permission to it");
                if !principals.contains_key(&sub) {
                    principals.insert(sub.clone(), Principal::new(sub.clone()));
                }

                let principal = principals.get_mut(&sub).unwrap();
                principal.add_permission(permission);
            }
        }

        log::debug!("Loading roles for principals");
        let role_assignments = self
            .db
            .list_all_partitioned::<RoleAssignment>("role")
            .await?;
        for row in role_assignments.rows {
            let assignment = &row.doc.unwrap();

            log::debug!("Processing role assignment {:?}", assignment);
            let sub = assignment.subject().to_string();
            if !principals.contains_key(&sub) {
                principals.insert(sub.clone(), Principal::new(sub.clone()));
            }

            let principal = principals.get_mut(&sub).unwrap();
            log::debug!(
                "Found assignment subject {:?}. Adding role to it",
                principal
            );
            let assignment_role = assignment.role();
            let role = roles.get(assignment_role);
            let role = match role {
                Some(role) => role.clone(),
                None => Role::new(assignment_role.to_string()),
            };
            principal.add_role(role);
        }

        model.set_principals(principals);
        log::debug!("Finished loading rules");
        Ok(())
    }

    #[tracing::instrument]
    pub fn check(&self, sub: &Guid, obj: &Guid, act: &str) -> Result<(), EvaluationError> {
        if sub.to_string() == ROOT_USER {
            return Ok(());
        }

        let sub = &sub.to_string();
        let obj = &obj.to_string();
        log::info!(
            "Evaluating permission sub: {}, obj: {}, act: {}",
            sub,
            obj,
            act
        );
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

    #[tracing::instrument]
    pub async fn add_permission(&self, sub: Guid, obj: Guid, act: &str) -> Result<(), Error> {
        if sub.to_string() == ROOT_USER {
            return Ok(());
        }

        log::debug!(
            "adding permission sub: {}, obj: {}, act: {}",
            &sub,
            &obj,
            act
        );

        let rule = Rule::new(sub, obj, act.to_string());
        match self.db.put(&rule.id().to_string(), rule).await {
            Ok(_) => Ok(()),
            Err(err) => match err.status() {
                StatusCode::CONFLICT => Ok(()),
                _ => Err(Error::from(err)),
            },
        }
    }

    #[tracing::instrument]
    pub async fn remove_permission(&self, sub: &Guid, obj: Guid, act: &str) -> Result<(), Error> {
        if sub.to_string() == ROOT_USER {
            return Ok(());
        }

        let sub_name = sub.to_string();
        log::debug!("Removing permission form sub {}", &sub_name);
        let id = Rule::build_id(&sub_name, &obj.to_string(), &act.to_string());
        let rule = self
            .db
            .get::<Rule>(&id.to_string())
            .await?
            .ok_or_else(|| Error::not_found(id.partition().unwrap_or("permission"), id.id()))?;
        log::debug!("Permission found, removing");
        self.db
            .delete(&rule.id().to_string(), &rule.rev().unwrap())
            .await?;
        Ok(())
    }

    #[tracing::instrument]
    pub async fn list_principal_permissions(
        &self,
        sub: &Guid,
        limit: usize,
        offset: usize,
    ) -> Result<Page<Rule>, Error> {
        log::debug!(
            "Listing principal permissions for sub {} with limit: {} and offset: {}",
            &sub,
            limit,
            offset,
        );
        let response = self
            .db
            .find_partitioned::<Rule>(
                "rule",
                serde_json::json!({
                    "sub": sub.to_string(),
                }),
                limit,
                offset,
            )
            .await?;

        let total = self.db.count_partitioned("rule").await?;

        if let Some(warning) = &response.warning {
            log::warn!("{}", warning);
        }

        Ok(Page::from_find_response(response, limit, offset, total))
    }

    #[tracing::instrument]
    pub async fn add_role_to_principal(&self, sub: Guid, role: &str) -> Result<(), Error> {
        if sub.to_string() == ROOT_USER {
            return Ok(());
        }

        let assignment = RoleAssignment::new(sub, role.to_string());
        match self.db.put(&assignment.id().to_string(), assignment).await {
            Ok(_) => Ok(()),
            Err(err) => match err.status() {
                StatusCode::CONFLICT => Ok(()),
                _ => Err(Error::from(err)),
            },
        }
    }

    #[tracing::instrument]
    pub async fn remove_role_from_principal(&self, sub: &Guid, role: &str) -> Result<(), Error> {
        if sub.to_string() == ROOT_USER {
            return Ok(());
        }

        if let Some(assignment) = self
            .db
            .get::<RoleAssignment>(&RoleAssignment::build_id(&sub.to_string(), role).to_string())
            .await?
        {
            self.db
                .delete(&assignment.id().to_string(), &assignment.rev().unwrap())
                .await?;
        }

        Ok(())
    }

    #[tracing::instrument]
    pub async fn list_principal_roles(
        &self,
        sub: &Guid,
        limit: usize,
        offset: usize,
    ) -> Result<Page<String>, Error> {
        let response = self
            .db
            .find_partitioned::<RoleAssignment>(
                "role",
                serde_json::json!({
                    "subject": sub.to_string()
                }),
                limit,
                offset,
            )
            .await?;

        let total = self.db.count_partitioned("role").await?;

        if let Some(warning) = &response.warning {
            log::warn!("{}", warning);
        }

        let page = Page::from_find_response(response, limit, offset, total)
            .map(|assignment| assignment.role().to_string());
        Ok(page)
    }
}

#[derive(Debug)]
pub enum EvaluationError {
    Denied,
}

impl Display for EvaluationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Denied => write!(f, "Access denied"),
        }
    }
}

impl std::error::Error for EvaluationError {}
