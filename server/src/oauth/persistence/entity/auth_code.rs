use chrono::{DateTime, Utc};
use chrono::serde::ts_seconds;
use serde::{Deserialize, Serialize};

use crate::guid::Guid;
use crate::oauth::code::AuthorizationCode;
use crate::oauth::session::Session;
use crate::secure::SecureSecret;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuthorizationCodeEntity {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    session: Session,
    #[serde(with = "ts_seconds")]
    expiration: DateTime<Utc>,
}

impl AuthorizationCodeEntity {
    pub fn build_guid(id: &str) -> Guid {
        Guid::from(format!("code:{}", id))
    }
    pub fn new(sig: String, session: Session, expiration: DateTime<Utc>) -> AuthorizationCodeEntity {
        let id = Self::build_guid(&sig);
        AuthorizationCodeEntity { id, rev: None::<String>, session, expiration, }
    }

    pub fn id(&self) -> &Guid {
        &self.id
    }

    pub fn rev(&self) -> Option<String> {
        self.rev.clone()
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn to_empty_code(&self) -> AuthorizationCode {
        let expires_in = self.expiration.signed_duration_since(Utc::now());
        AuthorizationCode::new(SecureSecret::empty(), self.session().clone(), expires_in)
    }
}
