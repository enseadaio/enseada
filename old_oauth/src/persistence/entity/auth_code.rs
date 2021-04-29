use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use enseada::couchdb::repository::Entity;
use enseada::guid::Guid;
use enseada::secure::SecureSecret;

use crate::code::AuthorizationCode;
use crate::request::{PkceRequest, TransformationMethod};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pkce: Option<PkceRequestEntity>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PkceRequestEntity {
    code_challenge: String,
    code_challenge_method: TransformationMethod,
}

impl From<PkceRequest> for PkceRequestEntity {
    fn from(req: PkceRequest) -> Self {
        Self {
            code_challenge: req.code_challenge().to_string(),
            code_challenge_method: req.code_challenge_method().clone(),
        }
    }
}

impl Into<PkceRequest> for PkceRequestEntity {
    fn into(self) -> PkceRequest {
        PkceRequest::new(self.code_challenge, self.code_challenge_method)
    }
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
        pkce: Option<PkceRequestEntity>,
    ) -> AuthorizationCodeEntity {
        let id = Self::build_guid(&sig);
        AuthorizationCodeEntity {
            id,
            rev: None::<String>,
            session,
            expiration,
            pkce,
        }
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn pkce(&self) -> Option<&PkceRequestEntity> {
        self.pkce.as_ref()
    }

    pub fn into_anonymous_code(self) -> AuthorizationCode {
        let expires_in = self.expiration.signed_duration_since(Utc::now());
        AuthorizationCode::new(
            SecureSecret::empty(),
            self.session,
            expires_in,
            self.pkce.map(Into::into),
        )
    }
}
