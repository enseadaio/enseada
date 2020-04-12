use std::ops::Add;

use chrono::{DateTime, Duration, Utc};

use crate::oauth::Expirable;
use crate::oauth::scope::Scope;
use crate::oauth::session::Session;
use crate::secure::SecureSecret;

pub trait Token: ToString + Expirable {
    fn token(&self) -> SecureSecret;
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

    pub fn session(&self) -> &Session {
        &self.session
    }
}

impl Token for AccessToken {
    fn token(&self) -> SecureSecret {
        self.token_rep.clone().unwrap_or_else(SecureSecret::empty)
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
        self.expiration.signed_duration_since(Utc::now()).num_seconds()
    }

    fn is_expired(&self) -> bool {
        self.expiration.lt(&Utc::now())
    }
}

pub struct RefreshToken {
    token_rep: Option<SecureSecret>,
    session: Session,
    expiration: DateTime<Utc>,
}

impl RefreshToken {
    pub fn new(token: SecureSecret, session: Session, expires_in: Duration) -> RefreshToken {
        RefreshToken {
            token_rep: Some(token),
            session,
            expiration: Utc::now().add(expires_in),
        }
    }
    pub fn scope(&self) -> &Scope {
        &self.session.scope()
    }

    pub fn session(&self) -> &Session {
        &self.session
    }
}

impl Token for RefreshToken {
    fn token(&self) -> SecureSecret {
        self.token_rep.clone().unwrap_or_else(SecureSecret::empty)
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
        self.expiration.signed_duration_since(Utc::now()).num_seconds()
    }

    fn is_expired(&self) -> bool {
        self.expiration.lt(&Utc::now())
    }
}