use crate::oauth::Scope;


pub trait Token: ToString {
    fn token(&self) -> String;
}

pub struct AccessToken {
    token_rep: Option<String>,
    scope: Scope,
    expires_in: u16,
}

impl AccessToken {
    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn expires_in(&self) -> &u16 {
        &self.expires_in
    }
}

impl Token for AccessToken {
    fn token(&self) -> String {
        self.token_rep.clone().unwrap_or_else(|| "".to_string())
    }
}

impl ToString for AccessToken {
    fn to_string(&self) -> String {
        self.token()
    }
}

pub struct RefreshToken {
    token_rep: Option<String>,
    scope: Scope,
    expires_in: u16,
}

impl RefreshToken {
    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn expires_in(&self) -> &u16 {
        &self.expires_in
    }
}

impl Token for RefreshToken {
    fn token(&self) -> String {
        self.token_rep.clone().unwrap_or_else(|| "".to_string())
    }
}

impl ToString for RefreshToken {
    fn to_string(&self) -> String {
        self.token()
    }
}
