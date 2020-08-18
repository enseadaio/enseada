use std::ops::Add;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use enseada::secure::SecureSecret;

use crate::scope::Scope;
use crate::session::Session;
use crate::Expirable;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenTypeHint {
    AccessToken,
    RefreshToken,
    #[serde(other)]
    Unknown,
}

pub trait Token: ToString + Expirable {
    fn token(&self) -> SecureSecret;
    fn session(&self) -> &Session;
    fn type_hint(&self) -> TokenTypeHint;
}

pub struct AccessToken {
    token_rep: Option<SecureSecret>,
    session: Session,
    expiration: DateTime<Utc>,
}

impl AccessToken {
    pub fn new(token: SecureSecret, session: Session, expires_in: Duration) -> AccessToken {
        AccessToken {
            token_rep: Some(token),
            session,
            expiration: Utc::now().add(expires_in),
        }
    }

    pub fn scope(&self) -> &Scope {
        &self.session.scope()
    }
}

impl Token for AccessToken {
    fn token(&self) -> SecureSecret {
        self.token_rep.clone().unwrap_or_else(SecureSecret::empty)
    }

    fn session(&self) -> &Session {
        &self.session
    }

    fn type_hint(&self) -> TokenTypeHint {
        TokenTypeHint::AccessToken
    }
}

impl ToString for AccessToken {
    fn to_string(&self) -> String {
        self.token().to_string()
    }
}

impl Expirable for AccessToken {
    fn expiration(&self) -> &DateTime<Utc> {
        &self.expiration
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

pub struct RefreshToken {
    token_rep: Option<SecureSecret>,
    session: Session,
    expiration: DateTime<Utc>,
    related_access_token_signature: String,
}

impl RefreshToken {
    pub fn new(
        token: SecureSecret,
        session: Session,
        expires_in: Duration,
        related_access_token_signature: String,
    ) -> RefreshToken {
        RefreshToken {
            token_rep: Some(token),
            session,
            expiration: Utc::now().add(expires_in),
            related_access_token_signature,
        }
    }
    pub fn scope(&self) -> &Scope {
        &self.session.scope()
    }

    pub fn related_access_token_signature(&self) -> &str {
        &self.related_access_token_signature
    }
}

impl Token for RefreshToken {
    fn token(&self) -> SecureSecret {
        self.token_rep.clone().unwrap_or_else(SecureSecret::empty)
    }

    fn session(&self) -> &Session {
        &self.session
    }

    fn type_hint(&self) -> TokenTypeHint {
        TokenTypeHint::RefreshToken
    }
}

impl ToString for RefreshToken {
    fn to_string(&self) -> String {
        self.token().to_string()
    }
}

impl Expirable for RefreshToken {
    fn expiration(&self) -> &DateTime<Utc> {
        &self.expiration
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
