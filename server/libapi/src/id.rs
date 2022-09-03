use schemars::gen::SchemaGenerator;
use schemars::schema::{InstanceType, Schema, SchemaObject};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use ulid::Ulid;

#[derive(Debug, Default, PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Id(Ulid);

impl Id {
    pub fn new() -> Self {
        Ulid::new().into()
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Ulid::fmt(&self.0, f)
    }
}

impl From<Ulid> for Id {
    fn from(id: Ulid) -> Self {
        Self(id)
    }
}

impl FromStr for Id {
    type Err = ulid::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = Ulid::from_str(s)?.into();
        Ok(id)
    }
}

impl TryFrom<String> for Id {
    type Error = ulid::DecodeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = Ulid::deserialize(deserializer)?;
        Ok(id.into())
    }
}

impl JsonSchema for Id {
    fn schema_name() -> String {
        "Id".to_string()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            format: Some("ulid".to_string()),
            ..Default::default()
        }
        .into()
    }
}
