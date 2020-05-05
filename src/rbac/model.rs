use std::cmp::{Eq, PartialEq};
use std::collections::{HashMap, HashSet};

use glob::Pattern;

use crate::error::Error;

pub struct Model {
    principals: HashMap<String, Principal>,
}

impl Model {
    pub fn empty() -> Self {
        Model { principals: HashMap::new() }
    }

    pub fn set_principals(&mut self, principals: HashMap<String, Principal>) -> &mut Self {
        self.principals = principals;
        self
    }

    pub fn add_principal(&mut self, principal: Principal) -> &mut Self {
        self.principals.insert(principal.name.clone(), principal);
        self
    }

    fn new(principals: Vec<Principal>) -> Self {
        let mut map = HashMap::new();
        for principal in principals {
            map.insert(principal.name.clone(), principal);
        }
        Model { principals: map }
    }

    pub fn check(&self, principal: &str, object: &str, action: &str) -> EvaluationResult {
        log::debug!("{:?}", &self.principals);
        if principal == "user:root" {
            return EvaluationResult::Granted;
        }

        let target_permission = Permission::new(object, action);
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

#[derive(Debug)]
pub struct Principal {
    name: String,
    roles: HashMap<String, Role>,
    permissions: HashSet<Permission>,
}

impl Principal {
    pub fn new(name: String) -> Self {
        Principal { name, roles: HashMap::new(), permissions: HashSet::new() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_role(&mut self, role: Role) -> &mut Self {
        self.roles.insert(role.name.clone(), role);
        self
    }

    pub fn add_permission(&mut self, permission: Permission) -> &mut Self {
        self.permissions.insert(permission);
        self
    }
}

impl Visitable for Principal {
    fn visit(&self, visitor: &Visitor) -> EvaluationResult {
        log::trace!("Visiting principal {}", &self.name);
        if self.permissions.contains(&visitor.target_permission) {
            log::trace!("Principal {} has exact matching permission", &self.name);
            return EvaluationResult::Granted;
        }

        for permission in &self.permissions {
            if permission.matches(&visitor.target_permission) {
                log::trace!("Principal {} has matching permission obj: {}, act: {}", &self.name, &permission.object, &permission.action);
                return EvaluationResult::Granted;
            }
        }

        for role in self.roles.values() {
            if role.visit(visitor) == EvaluationResult::Granted {
                return EvaluationResult::Granted;
            }
        }
        EvaluationResult::Denied
    }
}

#[derive(Debug, Clone)]
pub struct Role {
    name: String,
    permissions: HashSet<Permission>,
}

impl Role {
    pub fn new(name: String) -> Self {
        Role { name, permissions: HashSet::new() }
    }

    pub fn add_permission(&mut self, permission: Permission) -> &mut Self {
        self.permissions.insert(permission);
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Visitable for Role {
    fn visit(&self, visitor: &Visitor) -> EvaluationResult {
        log::trace!("Visiting role {}", &self.name);
        if self.permissions.contains(&visitor.target_permission) {
            log::trace!("Role {} has exact matching permission", &self.name);
            return EvaluationResult::Granted;
        }

        for permission in &self.permissions {
            if permission.matches(&visitor.target_permission) {
                log::trace!("Role {} has matching permission obj: {}, act: {}", &self.name, &permission.object, &permission.action);
                return EvaluationResult::Granted;
            }
        }

        EvaluationResult::Denied
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Permission {
    object: String,
    action: String,
    object_pattern: Pattern,
    action_pattern: Pattern,
}

impl Permission {
    pub fn new(object: &str, action: &str) -> Self {
        let object = object.to_string();
        let action = action.to_string();
        Permission {
            object_pattern: Pattern::new(&object).unwrap(),
            object,
            action_pattern: Pattern::new(&action).unwrap(),
            action,
        }
    }

    fn matches(&self, other: &Permission) -> bool {
        self.object_pattern.matches(&other.object)
            && self.action_pattern.matches(&other.action)
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
            permissions: vec![Permission::new("test", "test:*")].into_iter().collect(),
        };

        let model = Model::new(vec![principal]);
        let result = model.check("test", "test", "test:all");
        assert_eq!(result, EvaluationResult::Granted);
    }

    #[test]
    fn it_grants_role_permissions() {
        let role = Role {
            name: "test_role".to_string(),
            permissions: vec![Permission::new("test:*", "test")].into_iter().collect(),
        };

        let mut roles = HashMap::new();
        roles.insert(role.name.clone(), role);

        let principal = Principal {
            name: "test".to_string(),
            roles,
            permissions: HashSet::new(),
        };

        let model = Model::new(vec![principal]);
        let result = model.check("test", "test:all", "test");
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

    #[test]
    fn it_matches_a_matching_permission() {
        let a = Permission::new("test:*", "test:*");
        let b = Permission::new("test:hello", "test:world");

        assert!(a.matches(&b));
    }

    #[test]
    fn it_matches_an_identical_permission() {
        let a = Permission::new("test:hello", "test:world");
        let b = Permission::new("test:hello", "test:world");

        assert!(a.matches(&b));
    }

    #[test]
    fn it_does_not_match_a_non_matching_permission() {
        let a = Permission::new("test:*", "test:*");
        let b = Permission::new("other:hello", "other:world");

        assert!(!a.matches(&b));
    }

    #[test]
    fn it_matches_everything() {
        let a = Permission::new("*", "*");
        let b = Permission::new("test:hello", "test:world");

        assert!(a.matches(&b));
    }
}