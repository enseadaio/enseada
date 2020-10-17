use std::fmt::Display;

use serde::export::Formatter;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, PartialEq)]
pub struct Guid {
    partition: Option<String>,
    id: String,
    value: String,
}

impl Guid {
    pub fn simple<S: ToString>(id: S) -> Self {
        let id = id.to_string();
        let value = id.to_string();
        Guid {
            partition: None,
            id,
            value,
        }
    }

    pub fn partitioned<P: ToString, I: ToString>(partition: P, id: I) -> Self {
        let partition = partition.to_string();
        let id = id.to_string();
        Guid {
            value: format!("{}:{}", partition, id),
            partition: Some(partition),
            id,
        }
    }

    pub fn partition(&self) -> Option<&str> {
        self.partition.as_deref()
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

impl Display for Guid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.value)
    }
}

impl From<String> for Guid {
    fn from(s: String) -> Self {
        if s.contains(':') {
            let p: Vec<&str> = s.splitn(2, ':').collect();
            let partition = p.get(0).unwrap();
            let id = p.get(1).unwrap();
            Guid::partitioned(partition, id)
        } else {
            Guid::simple(s)
        }
    }
}

impl<'a> From<&'a str> for Guid {
    fn from(s: &'a str) -> Self {
        Guid::from(s.to_string())
    }
}

impl Serialize for Guid {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        self.value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Guid {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Guid::from(s))
    }
}

impl AsRef<[u8]> for Guid {
    fn as_ref(&self) -> &[u8] {
        self.value.as_bytes()
    }
}

impl AsRef<str> for Guid {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

#[cfg(test)]
mod test {
    use crate::guid::Guid;

    #[test]
    fn it_converts_from_string_with_partition() {
        let s = String::from("part:id:id");
        let guid = Guid::from(s.as_str());
        assert_eq!(guid.partition(), Some("part"));
        assert_eq!(guid.id(), &String::from("id:id"));
    }

    #[test]
    fn it_converts_from_string_without_partition() {
        let s = String::from("no-partition");
        let guid = Guid::from(s.as_str());
        assert_eq!(guid.partition(), None);
        assert_eq!(guid.id(), &s);
    }

    #[test]
    fn it_converts_from_string_and_to_string() {
        let s = String::from("part:id");
        let guid = Guid::from(s.as_str());
        assert_eq!(guid.to_string(), s);
    }

    #[test]
    fn it_serializes() {
        let s = String::from("part:id");
        let guid = Guid::from(s.as_str());
        let json = serde_json::to_string(&guid).unwrap();
        assert_eq!(json, format!("\"{}\"", &s));
    }

    #[test]
    fn it_deserializes() {
        let s = String::from("part:id");
        let guid: Guid = serde_json::from_str("\"part:id\"").unwrap();

        assert_eq!(guid.partition(), Some("part"));
        assert_eq!(guid.id(), &String::from("id"));
        assert_eq!(guid.to_string(), s);
    }
}
