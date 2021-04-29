use crate::model::{Model, Permission, Principal, Role};
use std::collections::HashMap;

#[derive(Default)]
pub struct Enforcer {
    model: Model,
}

impl Enforcer {
    pub fn new() -> Self {
        Default::default()
    }

    pub(crate) fn load_rules(&mut self) {
        let model = &mut self.model;
        let mut principals = HashMap::new();
        let mut roles = HashMap::new();

        for row in rules.rows {
            let rule = &row.doc.unwrap();
            let permission = Permission::new(rule.object(), rule.action());
            let sub = rule.subject().id().to_string();
            if rule.subject().partition() == Some("role") {
                if !roles.contains_key(&sub) {
                    roles.insert(sub.clone(), Role::new(&sub));
                }
                let role = roles.get_mut(&sub).unwrap();
                role.add_permission(permission);
            } else {
                if !principals.contains_key(&sub) {
                    principals.insert(sub.clone(), Principal::new(&sub));
                }

                let principal = principals.get_mut(&sub).unwrap();
                principal.add_permission(permission);
            }
        }

        let role_assignments = self
            .db
            .list_all_partitioned::<RoleAssignment>(
                RoleAssignment::build_guid("").partition().unwrap(),
            )
            .await?;
        for row in role_assignments.rows {
            let assignment = &row.doc.unwrap();

            let sub = assignment.subject().to_string();
            if !principals.contains_key(&sub) {
                principals.insert(sub.clone(), Principal::new(&sub));
            }

            let principal = principals.get_mut(&sub).unwrap();
            let assignment_role = assignment.role();
            let role = roles.get(assignment_role);
            let role = match role {
                Some(role) => role.clone(),
                None => model::Role::new(assignment_role),
            };
            principal.add_role(role);
        }

        model.set_principals(principals);
        Ok(())
    }
}
