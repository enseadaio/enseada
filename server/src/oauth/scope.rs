use std::collections::HashSet;
use std::vec::Vec;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::oauth::error::{Error, ErrorKind};
use crate::oauth::Result;

#[derive(Clone, Default, Debug)]
pub struct Scope(HashSet<String>);

impl Scope {
    /// Returns the intersecting scope, or an InvalidScope error
    /// if no intersection is found.
    /// A full scope always matches everything
    pub fn matches(&self, other: &Scope) -> Result<Scope> {
        if self.is_full_scope() {
            return Ok(other.clone());
        }

        let intersection: HashSet<String> =
            self.0.intersection(&other.0).map(String::clone).collect();
        if intersection.is_empty() {
            Err(Error::new(
                ErrorKind::InvalidScope,
                "invalid scope".to_string(),
            ))
        } else {
            Ok(Scope::from(intersection))
        }
    }

    /// Checks if the scope is a subset of the other scope
    /// A full scope always matches everything
    pub fn matches_exactly(&self, other: &Scope) -> Result<()> {
        if self.is_full_scope() {
            return Ok(());
        }

        if self.is_subset(other) {
            Ok(())
        } else {
            Err(Error::new(
                ErrorKind::InvalidScope,
                "invalid scope".to_string(),
            ))
        }
    }

    /// Returns true if the scope is a superset of another, i.e., self contains at least all the values in other.
    pub fn is_superset(&self, other: &Scope) -> bool {
        self.is_full_scope() || self.0.is_superset(&other.0)
    }

    pub fn is_subset(&self, other: &Scope) -> bool {
        other.is_full_scope() || self.0.is_subset(&other.0)
    }

    /// Returns true if the scope is '*', meaning it matches every possible scope
    pub fn is_full_scope(&self) -> bool {
        self.0.contains("*")
    }
}

impl From<HashSet<String>> for Scope {
    fn from(set: HashSet<String>) -> Self {
        Scope(set)
    }
}

impl From<Vec<String>> for Scope {
    fn from(vec: Vec<String>) -> Self {
        let set = vec.iter().map(String::clone).collect();
        Scope(set)
    }
}

impl From<Vec<&str>> for Scope {
    fn from(vec: Vec<&str>) -> Self {
        Scope(vec.iter().map(|s| (*s).to_string()).collect())
    }
}

impl From<String> for Scope {
    fn from(scope: String) -> Self {
        Scope(scope.split(' ').map(|s| s.to_string()).collect())
    }
}

impl From<&str> for Scope {
    fn from(scope: &str) -> Self {
        Scope::from(scope.to_string())
    }
}

impl ToString for Scope {
    fn to_string(&self) -> String {
        let mut vec: Vec<String> = self.0.iter().map(String::clone).collect();
        vec.sort();
        vec.join(" ")
    }
}

impl Serialize for Scope {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Scope {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let scope = String::deserialize(deserializer)?;
        Ok(Scope(scope.split(' ').map(|s| s.to_string()).collect()))
    }
}

impl PartialEq for Scope {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

#[cfg(test)]
mod test {
    use super::super::error::ErrorKind;
    use super::Scope;

    #[test]
    fn it_matches_a_similar_scope() {
        let a = Scope::from("test profile email");
        let b = Scope::from("email test something");

        let i = a.matches(&b).unwrap();
        assert_eq!(i.to_string(), "email test");
    }

    #[test]
    fn it_does_not_match_a_different_scope() {
        let a = Scope::from("profile email");
        let b = Scope::from("test something");

        let i = a.matches(&b).unwrap_err();
        assert_eq!(i.kind(), &ErrorKind::InvalidScope);
        assert_eq!(i.to_string(), "\"invalid_scope\": invalid scope");
    }

    #[test]
    fn a_full_scope_always_matches_everything() {
        let a = Scope::from("*");
        let b = Scope::from("everything should match");
        assert!(a.is_full_scope());
        assert!(!b.is_full_scope());

        let i = a.matches(&b).unwrap();
        assert_eq!(i.to_string(), "everything match should");
        assert!(a.is_superset(&b));
        assert!(b.is_subset(&a));
    }

    #[test]
    fn it_checks_a_subset() {
        let a = Scope::from("profile email");
        let b = Scope::from("profile");

        assert!(a.is_superset(&b))
    }

    #[test]
    fn it_does_not_check_an_invalid_subset() {
        let a = Scope::from("profile email");
        let b = Scope::from("test");

        assert!(!a.is_superset(&b))
    }
}
