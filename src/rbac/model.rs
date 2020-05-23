use std::cmp::{Eq, PartialEq};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Display;

use glob::Pattern;
use serde::export::Formatter;

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
        if principal == "user:root" {
            return EvaluationResult::Granted;
        }

        let target_permission = Permission::new(object, action);
        let visitor = Visitor { target_permission };
        let principal = match self.principals.get(principal) {
            Some(principal) => principal,
            None => return EvaluationResult::Denied,
        };

        log::trace!("Found principal {}", &principal.name);
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
    permissions: HashMap<String, Permission>,
}

impl Principal {
    pub fn new(name: String) -> Self {
        Principal { name, roles: HashMap::new(), permissions: HashMap::new() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_role(&mut self, role: Role) -> &mut Self {
        self.roles.insert(role.name.clone(), role);
        self
    }

    pub fn add_permission(&mut self, permission: Permission) -> &mut Self {
        self.permissions.insert(permission.to_string(), permission);
        self
    }

    pub fn permissions(&self) -> HashSet<&Permission> {
        self.permissions.values().collect()
    }
}

impl Visitable for Principal {
    fn visit(&self, visitor: &Visitor) -> EvaluationResult {
        log::trace!("Visiting principal {}", &self.name);
        let target_permission = &visitor.target_permission;
        let permissions = self.permissions();
        if permissions.contains(target_permission) {
            log::trace!("Principal {} has exact matching permission", &self.name);
            return EvaluationResult::Granted;
        }

        for permission in permissions {
            log::trace!("Checking if permission '{}' matches target '{}'", permission, target_permission);
            if permission.matches(target_permission) {
                log::trace!("Principal {} has matching permission obj: {}, act: {}", &self.name, &permission.object, &permission.action);
                return EvaluationResult::Granted;
            }
            log::trace!("Permission doesn't match target. Continuing");
        }

        log::trace!("No permissions found for target. Checking roles");

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
    permissions: HashMap<String, Permission>,
}

impl Role {
    pub fn new(name: String) -> Self {
        Role { name, permissions: HashMap::new() }
    }

    pub fn add_permission(&mut self, permission: Permission) -> &mut Self {
        self.permissions.insert(permission.to_string(), permission);
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn permissions(&self) -> HashSet<&Permission> {
        self.permissions.values().collect()
    }
}

impl Visitable for Role {
    fn visit(&self, visitor: &Visitor) -> EvaluationResult {
        log::trace!("Visiting role {}", &self.name);
        log::trace!("{:?}", &self.permissions);
        let target_permission = &visitor.target_permission;
        let permissions = self.permissions();
        if permissions.contains(target_permission) {
            log::trace!("Role {} has exact matching permission", &self.name);
            return EvaluationResult::Granted;
        }

        for permission in permissions {
            log::trace!("Checking if permission '{}' matches target '{}'", permission, target_permission);
            if permission.matches(target_permission) {
                log::trace!("Role {} has matching permission obj: {}, act: {}", &self.name, &permission.object, &permission.action);
                return EvaluationResult::Granted;
            }
            log::trace!("Permission doesn't match target. Continuing.");
        }

        log::trace!("No permissions found for target. Checking roles");
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

impl Display for Permission {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", &self.object, &self.action)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_grants_direct_permissions() {
        let mut principal = Principal::new("test".to_string());
        principal.add_permission(Permission::new("test", "test:*"));

        let model = Model::new(vec![principal]);
        let result = model.check("test", "test", "test:all");
        assert_eq!(result, EvaluationResult::Granted);
    }

    #[test]
    fn it_grants_role_permissions() {
        let mut role = Role::new("test_role".to_string());
        role.add_permission(Permission::new("test:*", "test"));

        let mut principal = Principal::new("test".to_string());
        principal.add_role(role);

        let model = Model::new(vec![principal]);
        let result = model.check("test", "test:all", "test");
        assert_eq!(result, EvaluationResult::Granted);
    }

    #[test]
    fn it_doesnt_grant_missing_permissions() {
        let mut principal = Principal::new("test".to_string());
        principal.add_permission(Permission::new("test", "test"));

        let model = Model::new(vec![principal]);
        let result = model.check("test", "test", "another_test");
        assert_eq!(result, EvaluationResult::Denied);
    }

    #[test]
    fn it_doesnt_grant_permissions_to_missing_principal() {
        let mut principal = Principal::new("test".to_string());
        principal.add_permission(Permission::new("test", "test"));

        let model = Model::new(vec![principal]);
        let result = model.check("another_test", "test", "test");
        assert_eq!(result, EvaluationResult::Denied);
    }

    #[test]
    fn it_doesnt_grant_permissions_to_missing_object() {
        let mut principal = Principal::new("test".to_string());
        principal.add_permission(Permission::new("test", "test"));

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