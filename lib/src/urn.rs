use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};

use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub struct Urn {
    domain: String,
    namespace: Option<String>,
    kind: String,
    name: String,
}

#[derive(Default)]
pub struct UrnBuilder {
    domain: String,
    namespace: Option<String>,
    kind: String,
    name: String,
}

impl UrnBuilder {
    pub fn domain<S: Into<String>>(mut self, domain: S) -> Self {
        self.domain = domain.into();
        self
    }
    pub fn namespace<S: Into<String>>(mut self, namespace: S) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    pub fn kind<S: Into<String>>(mut self, kind: S) -> Self {
        self.kind = kind.into();
        self
    }
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = name.into();
        self
    }

    pub fn build(self) -> Urn {
        Urn {
            domain: self.domain,
            namespace: self.namespace,
            kind: self.kind,
            name: self.name,
        }
    }
}

impl Urn {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> UrnBuilder {
        Default::default()
    }

    pub fn domain(&self) -> &str {
        &self.domain
    }

    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl TryFrom<&str> for Urn {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parts: Vec<String> = value.split(':').map(str::to_ascii_lowercase).collect();
        let first = parts.get(0);
        if first.is_none() || first.unwrap() != "urn" {
            return Err(String::from("not an URN"));
        }

        let domain = parts
            .get(1)
            .cloned()
            .ok_or_else(|| "domain is missing".to_string())?;
        let namespace = parts.get(2).cloned();
        let kind = parts
            .get(3)
            .cloned()
            .ok_or_else(|| "kind is missing".to_string())?;
        let name = parts
            .get(4)
            .cloned()
            .ok_or_else(|| "name is missing".to_string())?;

        Ok(Urn {
            domain,
            namespace,
            kind,
            name,
        })
    }
}

impl TryFrom<&String> for Urn {
    type Error = String;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<String> for Urn {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl Display for Urn {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = format!(
            "urn:{}:{}:{}:{}",
            &self.domain,
            self.namespace.as_deref().unwrap_or_default(),
            &self.kind,
            &self.name
        );
        s.fmt(f)
    }
}

impl<'de> Deserialize<'de> for Urn {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Urn::try_from(s).map_err(D::Error::custom)
    }
}

impl Serialize for Urn {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let s = self.to_string();
        s.serialize(serializer)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_serializes_to_string() {
        let urn = Urn::new()
            .domain("test-domain")
            .namespace("ns")
            .kind("test-kind")
            .name("test-name")
            .build();
        let s = urn.to_string();
        assert_eq!("urn:test-domain:ns:test-kind:test-name", &s)
    }

    #[test]
    fn it_serializes_to_string_without_a_namespace() {
        let urn = Urn::new()
            .domain("test-domain")
            .kind("test-kind")
            .name("test-name")
            .build();
        let s = urn.to_string();
        assert_eq!("urn:test-domain::test-kind:test-name", &s)
    }

    #[test]
    fn it_serializes_to_json() {
        let urn = Urn::new()
            .domain("test-domain")
            .namespace("ns")
            .kind("test-kind")
            .name("test-name")
            .build();
        let j = serde_json::to_string(&urn).expect("to json failed");
        assert_eq!("\"urn:test-domain:ns:test-kind:test-name\"", &j)
    }

    #[test]
    fn it_serializes_to_json_without_a_namespace() {
        let urn = Urn::new()
            .domain("test-domain")
            .kind("test-kind")
            .name("test-name")
            .build();
        let j = serde_json::to_string(&urn).expect("to json failed");
        assert_eq!("\"urn:test-domain::test-kind:test-name\"", &j)
    }

    #[test]
    fn it_converts_from_a_string() {
        let res = Urn::try_from("urn:test-domain:ns:test-kind:test-name");
        assert!(res.is_ok());
        let urn = res.unwrap();
        assert_eq!("test-domain", urn.domain());
        assert_eq!(Some("ns"), urn.namespace());
        assert_eq!("test-kind", urn.kind());
        assert_eq!("test-name", urn.name());
    }

    #[test]
    fn it_converts_from_a_string_without_namespace() {
        let res = Urn::try_from("urn:test-domain::test-kind:test-name");
        assert!(res.is_ok());
        let urn = res.unwrap();
        assert_eq!("test-domain", urn.domain());
        assert_eq!(None, urn.namespace());
        assert_eq!("test-kind", urn.kind());
        assert_eq!("test-name", urn.name());
    }

    #[test]
    fn it_does_not_converts_from_a_string_with_wrong_prefix() {
        let res = Urn::try_from("arn:test-domain:ns:test-kind:test-name");
        assert!(res.is_err());
        assert_eq!("not an URN", &res.err().unwrap())
    }
}
