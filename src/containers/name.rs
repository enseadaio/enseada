use std::fmt::{self, Display, Formatter};

use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::containers::handler::NameParams;

lazy_static! {
    pub static ref REGEX: Regex = Regex::new("[a-z0-9]+(?:[._-][a-z0-9]+)*").unwrap();
}

pub fn is_valid_name(group: &str, name: &str) -> bool {
    REGEX.is_match(group) && REGEX.is_match(name)
}

#[derive(Clone, Debug)]
pub struct Name {
    group: String,
    name: String,
}

impl Name {
    pub fn new(group: String, name: String) -> Self {
        Name {
            group,
            name,
        }
    }

    pub fn group(&self) -> &str {
        &self.group
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl From<NameParams> for Name {
    fn from(params: NameParams) -> Self {
        Self::new(params.group, params.name)
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", &self.group, &self.name)
    }
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.split('/').collect();
        let group = parts.first().cloned().unwrap_or("");
        let name = parts.last().cloned().unwrap_or("");
        Ok(Name {
            group: group.to_string(),
            name: name.to_string(),
        })
    }
}

impl Serialize for Name {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {
        let s = self.to_string();
        s.serialize(serializer)
    }
}