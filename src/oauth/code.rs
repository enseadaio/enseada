use crate::secure::SecureSecret;
use std::convert::TryFrom;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::Error;
use crate::oauth::session::Session;

#[derive(Debug, Clone)]
pub struct AuthorizationCode {
    code: SecureSecret,
    session: Session,
}

impl AuthorizationCode {
    pub fn new(code: SecureSecret, session: Session) -> AuthorizationCode {
        AuthorizationCode { code, session }
    }

    pub fn session(&self) -> &Session {
        &self.session
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