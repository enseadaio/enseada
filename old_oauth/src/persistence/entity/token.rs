use chrono::serde::ts_seconds;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use enseada::couchdb::repository::Entity;
use enseada::guid::Guid;
use enseada::secure::SecureSecret;

use crate::session::Session;
use crate::token::{AccessToken, RefreshToken, Token};
use crate::Expirable;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AccessTokenEntity {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    session: Session,
    #[serde(with = "ts_seconds")]
    expiration: DateTime<Utc>,
}

impl Entity for AccessTokenEntity {
    fn build_guid(id: &str) -> Guid {
        Guid::from(format!("access_token:{}", id))
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

impl AccessTokenEntity {
    pub fn new(sig: String, session: Session, expiration: DateTime<Utc>) -> AccessTokenEntity {
        let id = Self::build_guid(&sig);
        AccessTokenEntity {
            id,
            rev: None::<String>,
            session,
            expiration,
        }
    }

    pub fn from_token(sig: String, token: &AccessToken) -> Self {
        Self::new(sig, token.session().clone(), token.expiration())
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn expiration(&self) -> DateTime<Utc> {
        self.expiration
    }

    pub fn expires_in(&self) -> Duration {
        self.expiration.signed_duration_since(Utc::now())
    }

    pub fn to_token(&self, token: SecureSecret) -> AccessToken {
        AccessToken::new(token, self.session.clone(), self.expiration())
    }

    pub fn to_empty_token(&self) -> AccessToken {
        self.to_token(SecureSecret::empty())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RefreshTokenEntity {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    session: Session,
    #[serde(with = "ts_seconds")]
    expiration: DateTime<Utc>,
    related_access_token_signature: String,
}

impl Entity for RefreshTokenEntity {
    fn build_guid(id: &str) -> Guid {
        Guid::from(format!("access_token:{}", id))
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

impl RefreshTokenEntity {
    pub fn new(
        sig: String,
        session: Session,
        expiration: DateTime<Utc>,
        related_access_token_signature: String,
    ) -> RefreshTokenEntity {
        let id = Self::build_guid(&sig);
        RefreshTokenEntity {
            id,
            rev: None,
            session,
            expiration,
            related_access_token_signature,
        }
    }

    pub fn from_token(sig: String, token: &RefreshToken) -> Self {
        Self::new(
            sig,
            token.session().clone(),
            token.expiration(),
            token.related_access_token_signature().to_string(),
        )
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn expiration(&self) -> DateTime<Utc> {
        self.expiration
    }

    pub fn expires_in(&self) -> Duration {
        self.expiration.signed_duration_since(Utc::now())
    }

    pub fn to_token(&self, token: SecureSecret) -> RefreshToken {
        RefreshToken::new(
            token,
            self.session.clone(),
            self.expiration(),
            self.related_access_token_signature.clone(),
        )
    }

    pub fn to_empty_token(&self) -> RefreshToken {
        self.to_token(SecureSecret::empty())
    }
}
