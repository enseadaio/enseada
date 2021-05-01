use std::collections::HashMap;

use api::{GroupKindName, KindNamedRef};
pub use reloader::EnforcerReloader;

use crate::api::v1alpha1::{Policy, PolicyAttachment, RoleAttachment, Rule};
use crate::error::Error;
use crate::model::{EvaluationResult, Model, Permission, Principal, Role};

mod reloader;

#[derive(Default)]
pub struct Enforcer {
    model: Model,
}

impl Enforcer {
    pub fn new() -> Self {
        Self::default()
    }

    fn load_rules(&mut self, policies: Vec<Policy>, policy_attachments: Vec<PolicyAttachment>, role_attachments: Vec<RoleAttachment>) {
        let mut model = &mut self.model;
        let mut principals = HashMap::new();
        let mut roles = HashMap::new();

        let policy_map: HashMap<String, Vec<Rule>> = policies.into_iter().map(|p| (p.metadata.name, p.rules)).collect();
        let attachments_map: HashMap<String, Vec<KindNamedRef>> = policy_attachments.into_iter().map(|a| (a.policy_ref.name, a.subjects)).collect();

        for (policy, rules) in &policy_map {
            for rule in rules {
                for resource in &rule.resources {
                    for action in &rule.actions {
                        if let Some(subjects) = attachments_map.get(policy) {
                            for KindNamedRef { name, kind } in subjects {
                                let permission = Permission::new(resource, action);
                                match kind.to_lowercase().as_str() {
                                    "role" => {
                                        let role = roles.entry(format!("role:{}", name)).or_insert_with_key(|name| Role::new(name));
                                        role.add_permission(permission);
                                    }
                                    "user" => {
                                        let principal = principals.entry(format!("user:{}", name)).or_insert_with_key(|name| Principal::new(name));
                                        principal.add_permission(permission);
                                    }
                                    other => panic!("unsupported policy subject {}", other) // TODO: handle
                                };
                            }
                        }
                    }
                }
            }
        }

        for (role_name, user_name) in role_attachments.into_iter().map(|a| (format!("role:{}", a.role_ref.name), format!("user:{}", a.user_ref.name))) {
            let principal = principals.entry(user_name).or_insert_with_key(|name| Principal::new(name));
            let role = roles.get(&role_name);
            let role = match role {
                Some(role) => role.clone(),
                None => Role::new(role_name),
            };
            principal.add_role(role);
        }

        model.set_principals(principals);
    }

    pub fn check(&self, sub: &KindNamedRef, obj: &GroupKindName, act: &str) -> Result<(), Error> {
        let sub = format!("{}:{}", sub.kind.to_lowercase(), sub.name);
        let obj  = obj.to_string();
        match self.model.check(&sub, &obj, act) {
            EvaluationResult::Granted => Ok(()),
            EvaluationResult::Denied => Err(Error::denied(format!("Access denied: {} cannot perform action '{}' on resource '{}'", sub, act, obj)))
        }
    }
}

#[cfg(test)]
mod test {
    use api::{GroupVersionKindName, KindNamedRef, NamedRef, Resource};
    use api::core::v1alpha1::Metadata;

    use crate::api::v1alpha1::{Policy, PolicyAttachment, RoleAttachment, Rule};

    use super::*;

    #[test]
    fn it_builds_a_valid_model() {
        let policy_1 = Policy {
            metadata: Metadata::named("test-1"),
            rules: vec![
                Rule {
                    resources: vec![GroupKindName::new("test", "Test", "*")],
                    actions: vec!["*".to_string()],
                }
            ],
            ..Default::default()
        };

        let policy_2 = Policy {
            metadata: Metadata::named("test-2"),
            rules: vec![
                Rule {
                    resources: vec![GroupKindName::new("test", "Test", "*")],
                    actions: vec!["*".to_string()],
                }
            ],
            ..Default::default()
        };

        let policy_attachments = vec![
            PolicyAttachment {
                type_meta: PolicyAttachment::type_meta(),
                metadata: Metadata::named("test-1"),
                policy_ref: policy_1.to_ref(),
                subjects: vec![
                    KindNamedRef {
                        kind: "User".to_string(),
                        name: "test".to_string(),
                    },
                ],
            },
            PolicyAttachment {
                type_meta: PolicyAttachment::type_meta(),
                metadata: Metadata::named("test-2"),
                policy_ref: policy_2.to_ref(),
                subjects: vec![
                    KindNamedRef {
                        kind: "Role".to_string(),
                        name: "test".to_string(),
                    },
                ],
            },
        ];

        let policies = vec![policy_1, policy_2];


        let role_attachments = vec![
            RoleAttachment {
                type_meta: RoleAttachment::type_meta(),
                metadata: Metadata::named("test-2"),
                role_ref: NamedRef { name: "test".to_string() },
                user_ref: NamedRef { name: "test".to_string() },
            },
        ];

        let mut enforcer = Enforcer::new();
        enforcer.load_rules(policies, policy_attachments, role_attachments);

        let user_ref = KindNamedRef {
            kind: "User".to_string(),
            name: "test".to_string(),
        };

        assert!(enforcer.check(&user_ref, &GroupKindName::new("test", "test", "test"), "read").is_ok());
        assert!(enforcer.check(&user_ref, &GroupKindName::new("test", "test", "test"), "read").is_ok());
        assert!(enforcer.check(&user_ref, &GroupKindName::new("test", "test", "test"), "read").is_err());
    }
}
