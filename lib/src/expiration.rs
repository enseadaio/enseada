use chrono::{DateTime, Utc};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

static INFINITE: &str = "infinite";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Expiration {
    DateTime(DateTime<Utc>),
    #[serde(serialize_with = "infinite_ser")]
    #[serde(deserialize_with = "infinite_der")]
    Infinite,
}

fn infinite_ser<S>(serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    INFINITE.to_string().serialize(serializer)
}

fn infinite_der<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s == INFINITE {
        Ok(())
    } else {
        Err(D::Error::custom(format!("'{}' is not 'infinite'", s)))
    }
}

impl From<DateTime<Utc>> for Expiration {
    fn from(dt: DateTime<Utc>) -> Self {
        if dt == chrono::MAX_DATETIME {
            Self::Infinite
        } else {
            Self::DateTime(dt)
        }
    }
}

impl Into<DateTime<Utc>> for Expiration {
    fn into(self) -> DateTime<Utc> {
        match self {
            Expiration::DateTime(dt) => dt,
            Expiration::Infinite => chrono::MAX_DATETIME,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_serializes_date_time() {
        let now = Utc::now();
        let exp = Expiration::DateTime(now);
        let exp_json = serde_json::to_string(&exp).unwrap();
        let now_json = serde_json::to_string(&now).unwrap();
        assert_eq!(exp_json, now_json)
    }

    #[test]
    fn it_serializes_infinite() {
        let exp = Expiration::Infinite;
        let exp_json = serde_json::to_string(&exp).unwrap();
        println!("{}", &exp_json);
    }
}
