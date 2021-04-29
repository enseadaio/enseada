use std::fmt::{self, Display, Formatter};

use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, PartialOrd, PartialEq)]
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

#[derive(Clone, Debug, PartialOrd)]
pub struct GroupVersionKind {
    gv: GroupVersion,
    kind: String,
}

impl GroupVersionKind {
    pub fn new<G: ToString, V: ToString, K: ToString>(group: G, version: V, kind: K) -> Self {
        Self {
            gv: GroupVersion {
                group: group.to_string(),
                version: version.to_string(),
            },
            kind: kind.to_string(),
        }
    }
}

impl<'de> Deserialize<'de> for GroupVersionKind {
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
                kind: kind.to_string(),
            })
        } else {
            Err(<D as Deserializer<'de>>::Error::custom(format!(
                "invalid GroupVersionKind '{}'",
                s
            )))
        }
    }
}

impl Serialize for GroupVersionKind {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
        where
            S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl Display for GroupVersionKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.gv, self.kind.to_lowercase())
    }
}

impl PartialEq for GroupVersionKind {
    fn eq(&self, other: &Self) -> bool {
        (self.gv == other.gv) && (self.kind.to_lowercase() == other.kind.to_lowercase())
    }
}

#[derive(Clone, Debug, PartialOrd)]
pub struct GroupVersionKindName {
    gvk: GroupVersionKind,
    name: String,
}

impl GroupVersionKindName {
    pub fn new<G: ToString, V: ToString, K: ToString, N: ToString>(group: G, version: V, kind: K, name: N) -> Self {
        Self {
            gvk: GroupVersionKind::new(group, version, kind),
            name: name.to_string(),
        }
    }
}

impl<'de> Deserialize<'de> for GroupVersionKindName {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
        where
            D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.split('/').collect();

        if let [group, version, kind, name, ..] = parts.as_slice() {
            Ok(Self::new(group, version, kind, name))
        } else {
            Err(<D as Deserializer<'de>>::Error::custom(format!(
                "invalid GroupVersionKindName '{}'",
                s
            )))
        }
    }
}

impl Serialize for GroupVersionKindName {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
        where
            S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl Display for GroupVersionKindName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.gvk, self.name.to_lowercase())
    }
}

impl PartialEq for GroupVersionKindName {
    fn eq(&self, other: &Self) -> bool {
        (self.gvk == other.gvk) && (self.name.to_lowercase() == other.name.to_lowercase())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_serializes_a_group_version() {
        let gv = GroupVersion {
            group: "test".to_string(),
            version: "v1".to_string(),
        };

        let gv_s = serde_json::to_string(&gv).unwrap();
        assert_eq!("\"test/v1\"", gv_s);
    }

    #[test]
    fn it_deserializes_a_group_version() {
        let gv = GroupVersion {
            group: "test".to_string(),
            version: "v1".to_string(),
        };

        let gv_s = serde_json::from_str("\"test/v1\"").unwrap();
        assert_eq!(gv, gv_s);
    }

    #[test]
    fn it_serializes_a_group_version_kind() {
        let gvk = GroupVersionKind::new("test", "v1", "Test");

        let gvk_s = serde_json::to_string(&gvk).unwrap();
        assert_eq!("\"test/v1/test\"", gvk_s);
    }

    #[test]
    fn it_deserializes_a_group_version_kind() {
        let gvk = GroupVersionKind::new("test", "v1", "Test");

        let gvk_s = serde_json::from_str("\"test/v1/test\"").unwrap();
        assert_eq!(gvk, gvk_s);
    }

    #[test]
    fn it_serializes_a_group_version_kind_name() {
        let gvkn = GroupVersionKindName::new("test", "v1", "Test", "test");

        let gvkn_s = serde_json::to_string(&gvkn).unwrap();
        assert_eq!("\"test/v1/test/test\"", gvkn_s);
    }

    #[test]
    fn it_deserializes_a_group_version_kind_name() {
        let gvkn = GroupVersionKindName::new("test", "v1", "Test", "test");

        let gvkn_s = serde_json::from_str("\"test/v1/test/test\"").unwrap();
        assert_eq!(gvkn, gvkn_s);
    }
}
