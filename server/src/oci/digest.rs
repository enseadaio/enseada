use std::fmt::{self, Display, Formatter};

use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug)]
pub struct Digest {
    algo: String,
    digest: String,
}

impl Digest {
    pub fn sha256(digest: String) -> Self {
        Digest {
            algo: "sha256".to_string(),
            digest,
        }
    }
}

impl Display for Digest {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", &self.algo, &self.digest)
    }
}

impl<'de> Deserialize<'de> for Digest {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.split(':').collect();
        let algo = parts.first().cloned().unwrap_or("");
        let digest = parts.last().cloned().unwrap_or("");
        match algo {
            "sha256" => Ok(Digest::sha256(digest.to_string())),
            _ => Err(D::Error::custom(format!(
                "invalid digest algorithm '{}'",
                algo
            ))),
        }
    }
}

impl Serialize for Digest {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let s = self.to_string();
        s.serialize(serializer)
    }
}
