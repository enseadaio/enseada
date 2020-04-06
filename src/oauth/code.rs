use crate::secure::SecureSecret;
use std::convert::TryFrom;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::Error;

#[derive(Debug, Clone)]
pub struct AuthorizationCode {
    code: SecureSecret
}

impl TryFrom<String> for AuthorizationCode {
    type Error = hex::FromHexError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let code = hex::decode(s)?;
        Ok(AuthorizationCode { code: SecureSecret::new(code) })
    }
}

impl From<SecureSecret> for AuthorizationCode {
    fn from(code: SecureSecret) -> Self {
        AuthorizationCode { code }
    }
}

impl ToString for AuthorizationCode {
    fn to_string(&self) -> String {
        self.code.to_string()
    }
}

impl Serialize for AuthorizationCode {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {
        let code = self.code.to_string();
        code.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AuthorizationCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de> {
        let code = String::deserialize(deserializer)?;
        AuthorizationCode::try_from(code).map_err(serde::de::Error::custom)

    }
}