use serde::{Serialize, Deserialize, Deserializer, Serializer};
use std::vec::Vec;

#[derive(Clone, Default, Debug)]
pub struct Scope(Vec<String>);

impl From<Vec<String>> for Scope {
    fn from(vec: Vec<String>) -> Self {
        Scope(vec)
    }
}

impl From<Vec<&str>> for Scope {
    fn from(vec: Vec<&str>) -> Self {
        Scope(vec.iter().map(|s| s.to_string()).collect())
    }
}

impl From<String> for Scope {
    fn from(scope: String) -> Self {
        Scope(scope.split(' ')
            .map(|s| s.to_string())
            .collect())
    }
}

impl From<&str> for Scope {
    fn from(scope: &str) -> Self {
        Scope::from(scope.to_string())
    }
}

impl ToString for Scope {
    fn to_string(&self) -> String {
        self.0.join(" ")
    }
}

impl Serialize for Scope {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {
        self.0.join(" ").serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Scope {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de> {
        let scope = String::deserialize(deserializer)?;
        Ok(Scope(scope.split(" ").map(|s| s.to_string()).collect()))
    }
}