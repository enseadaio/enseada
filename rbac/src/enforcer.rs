use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::sync::Arc;

use http::StatusCode;

use enseada::couchdb::db::Database;
use enseada::couchdb::repository::{Entity, Repository};
use enseada::error::Error;
use enseada::guid::Guid;
use enseada::pagination::Page;

use crate::model::{self, EvaluationResult, Model, Permission, Principal};
use crate::role::Role;
use crate::rule::{RoleAssignment, Rule};
use crate::ROOT_USER;

#[derive(Debug)]
pub struct Enforcer {
    db: Database,
    model: Model,
}

impl Enforcer {
    pub fn new(db: Database) -> Self {
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
        let rules = self
            .db
            .list_all_partitioned::<Rule>(Rule::build_guid("").partition().unwrap())
            .await?;
        for row in rules.rows {
            let rule = &row.doc.unwrap();
            log::debug!("Processing rule {:?}", rule);
            let permission = Permission::new(rule.object(), rule.action());
            let sub = rule.subject().id().to_string();
            if rule.subject().partition() == Some("role") {
                log::debug!("Rule has a role subject. Adding permission to it");
                if !roles.contains_key(&sub) {
                    roles.insert(sub.clone(), model::Role::new(&sub));
                }
                let role = roles.get_mut(&sub).unwrap();
                role.add_permission(permission);
            } else {
                log::debug!("Rule has a principal subject. Adding permission to it");
                if !principals.contains_key(&sub) {
                    principals.insert(sub.clone(), Principal::new(&sub));
                }

                let principal = principals.get_mut(&sub).unwrap();
                principal.add_permission(permission);
            }
        }

        log::debug!("Loading roles for principals");
        let role_assignments = self
            .db
            .list_all_partitioned::<RoleAssignment>(
                RoleAssignment::build_guid("").partition().unwrap(),
            )
            .await?;
        for row in role_assignments.rows {
            let assignment = &row.doc.unwrap();

            log::debug!("Processing role assignment {:?}", assignment);
            let sub = assignment.subject().to_string();
            if !principals.contains_key(&sub) {
                principals.insert(sub.clone(), Principal::new(&sub));
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
                None => model::Role::new(assignment_role),
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
        match self.save(rule).await {
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
        let rule: Rule = self
            .find(id.id())
            .await?
            .ok_or_else(|| Error::not_found(id.partition().unwrap_or("permission"), id.id()))?;
        log::debug!("Permission found, removing");
        self.delete(&rule).await.map_err(Error::from)
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
        self.find_all(
            limit,
            offset,
            serde_json::json!({
                "sub": sub.to_string(),
            }),
        )
        .await
        .map_err(Error::from)
    }

    #[tracing::instrument]
    pub async fn add_role_to_principal(&self, sub: Guid, role: &str) -> Result<(), Error> {
        if sub.to_string() == ROOT_USER {
            return Ok(());
        }

        let role: Role = self
            .find(role)
            .await?
            .ok_or_else(|| Error::not_found("role", role))?;
        let assignment = RoleAssignment::new(sub, role.name());
        match self.save(assignment).await {
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

        let opt: Option<RoleAssignment> =
            self.find(RoleAssignment::build_id(sub, role).id()).await?;
        if let Some(assignment) = opt {
            self.delete(&assignment).await?;
        }

        Ok(())
    }

    #[tracing::instrument]
    pub async fn list_principal_roles(
        &self,
        sub: &Guid,
        limit: usize,
        offset: usize,
    ) -> Result<Page<Role>, Error> {
        let assignments: Page<RoleAssignment> = self
            .find_all(
                limit,
                offset,
                serde_json::json!({
                    "subject": sub.to_string()
                }),
            )
            .await?;

        let ids: Vec<String> = assignments
            .iter()
            .map(RoleAssignment::role)
            .map(Role::build_guid)
            .map(|id| id.to_string())
            .collect();

        let page = self
            .find_all(
                limit,
                offset,
                serde_json::json!({
                    "_id": {
                        "$in": ids
                    }
                }),
            )
            .await?;
        Ok(page)
    }
}

impl Repository<Rule> for Enforcer {
    fn db(&self) -> &Database {
        &self.db
    }
}

impl Repository<RoleAssignment> for Enforcer {
    fn db(&self) -> &Database {
        &self.db
    }
}

impl Repository<Role> for Enforcer {
    fn db(&self) -> &Database {
        &self.db
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
