use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use enseada::couchdb::repository::Entity;
use enseada::guid::Guid;
use enseada::secure::SecureSecret;

use crate::code::AuthorizationCode;
use crate::session::Session;

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

impl Entity for AuthorizationCodeEntity {
    fn build_guid(id: &str) -> Guid {
        Guid::from(format!("code:{}", id))
    }

    fn id(&self) -> &Guid {
        &self.id
    }

    fn rev(&self) -> Option<&str> {
        self.rev.as_deref()
    }

    fn set_rev(&mut self, rev: String) -> &mut Self {
        self.rev = Some(rev);
        self
    }
}

impl AuthorizationCodeEntity {
    pub fn new(
        sig: String,
        session: Session,
        expiration: DateTime<Utc>,
    ) -> AuthorizationCodeEntity {
        let id = Self::build_guid(&sig);
        AuthorizationCodeEntity {
            id,
            rev: None::<String>,
            session,
            expiration,
        }
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn to_empty_code(&self) -> AuthorizationCode {
        let expires_in = self.expiration.signed_duration_since(Utc::now());
        AuthorizationCode::new(SecureSecret::empty(), self.session().clone(), expires_in)
    }
}
