use crate::oauth::Scope;
use std::fmt::{self, Display, Formatter};

pub struct AccessToken {
    token: String,
    scope: Scope,
    expires_in: u16,
}

impl Display for AccessToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.token)
    }
}

pub struct RefreshToken {
    token: String,
    scope: Scope,
    expires_in: u16,
}

impl Display for RefreshToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.token)
    }
}