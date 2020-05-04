use std::cmp::{Eq, PartialEq};
use std::collections::{HashMap, HashSet};

pub struct Model {
    principals: HashMap<String, Principal>,
}

impl Model {
    fn new(principals: Vec<Principal>) -> Self {
        let mut map = HashMap::new();
        for principal in principals.into_iter() {
            map.insert(principal.name.clone(), principal);
        }
        Model { principals: map }
    }

    pub fn check(&self, principal: &str, object: &str, action: &str) -> EvaluationResult {
        let target_permission = Permission { object: object.to_string(), action: action.to_string() };
        let visitor = Visitor { target_permission };
        let principal = match self.principals.get(principal) {
            Some(principal) => principal,
            None => return EvaluationResult::Denied,
        };

        principal.visit(&visitor)
    }
}

#[derive(Debug, PartialEq)]
pub enum EvaluationResult {
    Granted,
    Denied,
}

struct Visitor {
    target_permission: Permission,
}

impl Visitor {
    pub fn new(target_permission: Permission) -> Self {
        Visitor { target_permission }
    }
}

trait Visitable {
    fn visit(&self, visitor: &Visitor) -> EvaluationResult;
}

struct Principal {
    name: String,
    roles: HashMap<String, Role>,
    permissions: HashSet<Permission>,
}

impl Visitable for Principal {
    fn visit(&self, visitor: &Visitor) -> EvaluationResult {
        if self.permissions.contains(&visitor.target_permission) {
            EvaluationResult::Granted
        } else {
            for (_, role) in self.roles.iter() {
                if role.visit(visitor) == EvaluationResult::Granted {
                    return EvaluationResult::Granted;
                }
            }
            EvaluationResult::Denied
        }
    }
}

struct Role {
    name: String,
    permissions: HashSet<Permission>,
}

impl Visitable for Role {
    fn visit(&self, visitor: &Visitor) -> EvaluationResult {
        if self.permissions.contains(&visitor.target_permission) {
            EvaluationResult::Granted
        } else {
            EvaluationResult::Denied
        }
    }
}

#[derive(PartialEq, Eq, Hash)]
struct Permission {
    object: String,
    action: String,
}

impl Permission {
    fn new(object: &str, action: &str) -> Self {
        Permission { object: object.to_string(), action: action.to_string() }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_grants_direct_permissions() {
        let principal = Principal {
            name: "test".to_string(),
            roles: HashMap::new(),
            permissions: vec![Permission::new("test", "test")].into_iter().collect(),
        };

        let model = Model::new(vec![principal]);
        let result = model.check("test", "test", "test");
        assert_eq!(result, EvaluationResult::Granted);
    }

    #[test]
    fn it_grants_role_permissions() {
        let role = Role {
            name: "test_role".to_string(),
            permissions: vec![Permission::new("test", "test")].into_iter().collect(),
        };

        let mut roles = HashMap::new();
        roles.insert(role.name.clone(), role);

        let principal = Principal {
            name: "test".to_string(),
            roles,
            permissions: HashSet::new(),
        };

        let model = Model::new(vec![principal]);
        let result = model.check("test", "test", "test");
        assert_eq!(result, EvaluationResult::Granted);
    }

    #[test]
    fn it_doesnt_grant_missing_permissions() {
        let principal = Principal {
            name: "test".to_string(),
            roles: HashMap::new(),
            permissions: vec![Permission::new("test", "test")].into_iter().collect(),
        };

        let model = Model::new(vec![principal]);
        let result = model.check("test", "test", "another_test");
        assert_eq!(result, EvaluationResult::Denied);
    }

    #[test]
    fn it_doesnt_grant_permissions_to_missing_principal() {
        let principal = Principal {
            name: "test".to_string(),
            roles: HashMap::new(),
            permissions: vec![Permission::new("test", "test")].into_iter().collect(),
        };

        let model = Model::new(vec![principal]);
        let result = model.check("another_test", "test", "test");
        assert_eq!(result, EvaluationResult::Denied);
    }

    #[test]
    fn it_doesnt_grant_permissions_to_missing_object() {
        let principal = Principal {
            name: "test".to_string(),
            roles: HashMap::new(),
            permissions: vec![Permission::new("test", "test")].into_iter().collect(),
        };

        let model = Model::new(vec![principal]);
        let result = model.check("test", "another_test", "test");
        assert_eq!(result, EvaluationResult::Denied);
    }
}