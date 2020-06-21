use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};

use serde::{de::Error as SerdeError, Deserialize, Deserializer, Serialize, Serializer};

use crate::oci::error::{Error, ErrorCode};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DigestAlgorithm {
    Sha256,
    Sha512,
}

impl Display for DigestAlgorithm {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            DigestAlgorithm::Sha256 => "sha256",
            DigestAlgorithm::Sha512 => "sha512",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug)]
pub struct Digest {
    algo: DigestAlgorithm,
    digest: String,
}

impl Digest {
    pub fn sha256(digest: String) -> Self {
        Digest {
            algo: DigestAlgorithm::Sha256,
            digest,
        }
    }

    pub fn sha512(digest: String) -> Self {
        Digest {
            algo: DigestAlgorithm::Sha512,
            digest,
        }
    }
}

impl Display for Digest {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", &self.algo, &self.digest)
    }
}

impl TryFrom<String> for Digest {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.split(':').collect();
        let algo = parts.first().cloned().unwrap_or("");
        let digest = parts.last().cloned().unwrap_or("");
        match algo {
            "sha256" => Ok(Digest::sha256(digest.to_string())),
            "sha512" => Ok(Digest::sha512(digest.to_string())),
            _ => Err(Error::new(
                ErrorCode::DigestInvalid,
                "provided digest algorithm not supported",
            )),
        }
    }
}

impl<'de> Deserialize<'de> for Digest {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::try_from(s).map_err(|err| D::Error::custom(err.to_string()))
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
