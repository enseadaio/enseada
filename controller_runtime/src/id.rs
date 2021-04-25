use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Error;

#[derive(Clone)]
pub struct Id {
    kind: String,
    name: String,
    value: String,
}

impl Id {
    pub fn new<K: ToString, N: ToString>(kind: K, name: N) -> Self {
        let kind = kind.to_string();
        let name = name.to_string();
        let value = format!("{}:{}", kind, name);
        Self {
            kind,
            name,
            value,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.value, f)
    }
}

impl AsRef<[u8]> for Id {
    fn as_ref(&self) -> &[u8] {
        self.value.as_bytes()
    }
}

impl TryFrom<String> for Id {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !value.contains(':') {
            return Err(format!("value '{}' is not a valid Resource ID", &value));
        }

        let p: Vec<&str> = value.splitn(2, ':').collect();
        let kind = p.get(0).unwrap();
        let name = p.get(1).unwrap();
        Ok(Self::new(kind, name))
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        Self::try_from(s).map_err(<D as Deserializer<'de>>::Error::custom)
    }
}

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {
        self.value.serialize(serializer)
    }
}
