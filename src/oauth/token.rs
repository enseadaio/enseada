use crate::oauth::scope::Scope;
use crate::secure::SecureSecret;
use crate::oauth::session::Session;

pub trait Token: ToString {
    fn token(&self) -> SecureSecret;
}

pub struct AccessToken {
    token_rep: Option<SecureSecret>,
    expires_in: u64,
    session: Session,
}

impl AccessToken {
    pub fn new(token: SecureSecret, session: Session, expires_in: u64) -> AccessToken {
        AccessToken {
            token_rep: Some(token),
            session,
            expires_in,
        }
    }

    pub fn scope(&self) -> &Scope {
        &self.session.scope()
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn expires_in(&self) -> &u64 {
        &self.expires_in
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

pub struct RefreshToken {
    token_rep: Option<SecureSecret>,
    session: Session,
    expires_in: u64,
}

impl RefreshToken {
    pub fn new(token: SecureSecret, session: Session, expires_in: u64) -> RefreshToken {
        RefreshToken {
            token_rep: Some(token),
            session,
            expires_in,
        }
    }
    pub fn scope(&self) -> &Scope {
        &self.session.scope()
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn expires_in(&self) -> &u64 {
        &self.expires_in
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
