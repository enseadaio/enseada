use std::ops::Add;

use chrono::{DateTime, Duration, Utc};
use serde::{Serialize, Serializer};

use crypto::SecureSecret;

use crate::request::PkceRequest;
use crate::session::Session;
use crate::Expirable;

#[derive(Debug, Clone)]
pub struct AuthorizationCode {
    code: SecureSecret,
    session: Session,
    expiration: DateTime<Utc>,
    pkce: Option<PkceRequest>,
}

impl AuthorizationCode {
    pub fn new(
        code: SecureSecret,
        session: Session,
        expiration: DateTime<Utc>,
        pkce: Option<PkceRequest>,
    ) -> AuthorizationCode {
        AuthorizationCode {
            code,
            session,
            expiration,
            pkce,
        }
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn pkce(&self) -> Option<&PkceRequest> {
        self.pkce.as_ref()
    }
}

impl Expirable for AuthorizationCode {
    fn expiration(&self) -> DateTime<Utc> {
        self.expiration
    }

    fn expires_in(&self) -> i64 {
        self.expiration
            .signed_duration_since(Utc::now())
            .num_seconds()
    }

    fn is_expired(&self) -> bool {
        self.expiration.lt(&Utc::now())
    }
}

impl ToString for AuthorizationCode {
    fn to_string(&self) -> String {
        self.code.to_string()
    }
}

impl Serialize for AuthorizationCode {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
        where
            S: Serializer,
    {
        let code = self.code.to_string();
        code.serialize(serializer)
    }
}
