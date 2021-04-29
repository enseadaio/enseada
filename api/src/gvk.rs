use std::fmt::{self, Display, Formatter};

use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug)]
pub struct GroupVersion {
    pub group: String,
    pub version: String,
}

impl<'de> Deserialize<'de> for GroupVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.split('/').collect();

        if let [group, version, ..] = parts.as_slice() {
            Ok(Self {
                group: group.to_string(),
                version: version.to_string(),
            })
        } else {
            Err(<D as Deserializer<'de>>::Error::custom(format!(
                "invalid GroupVersion '{}'",
                s
            )))
        }
    }
}

impl Serialize for GroupVersion {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl Display for GroupVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.group, self.version)
    }
}

#[derive(Clone, Debug)]
pub struct Gvk {
    gv: GroupVersion,
    kind: String,
    kind_plural: String,
}

impl<'de> Deserialize<'de> for Gvk {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.split('/').collect();

        if let [group, version, kind, ..] = parts.as_slice() {
            Ok(Self {
                gv: GroupVersion {
                    group: group.to_string(),
                    version: version.to_string(),
                },
                kind_plural: kind.to_string(),
            })
        } else {
            Err(<D as Deserializer<'de>>::Error::custom(format!(
                "invalid Gvk '{}'",
                s
            )))
        }
    }
}

impl Serialize for Gvk {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl Display for Gvk {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.group, self.version)
    }
}
