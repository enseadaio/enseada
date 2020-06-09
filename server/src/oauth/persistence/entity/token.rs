use chrono::serde::ts_seconds;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use enseada::guid::Guid;
use enseada::secure::SecureSecret;

use crate::oauth::session::Session;
use crate::oauth::token::{AccessToken, RefreshToken, Token};
use crate::oauth::Expirable;

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

impl AccessTokenEntity {
    pub fn build_guid(id: &str) -> Guid {
        Guid::from(format!("access_token:{}", id))
    }
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
        Self::new(sig, token.session().clone(), *token.expiration())
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

    pub fn expiration(&self) -> &DateTime<Utc> {
        &self.expiration
    }

    pub fn expires_in(&self) -> Duration {
        self.expiration.signed_duration_since(Utc::now())
    }

    pub fn to_token(&self, token: SecureSecret) -> AccessToken {
        AccessToken::new(token, self.session.clone(), self.expires_in())
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

impl RefreshTokenEntity {
    pub fn build_guid(id: &str) -> Guid {
        Guid::from(format!("refresh_token:{}", id))
    }

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
            *token.expiration(),
            token.related_access_token_signature().to_string(),
        )
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

    pub fn expiration(&self) -> &DateTime<Utc> {
        &self.expiration
    }

    pub fn expires_in(&self) -> Duration {
        self.expiration.signed_duration_since(Utc::now())
    }

    pub fn to_token(&self, token: SecureSecret) -> RefreshToken {
        RefreshToken::new(
            token,
            self.session.clone(),
            self.expires_in(),
            self.related_access_token_signature.clone(),
        )
    }

    pub fn to_empty_token(&self) -> RefreshToken {
        self.to_token(SecureSecret::empty())
    }
}
