use crate::oauth::{token, Scope};
use std::fmt::{self, Display};
use serde::export::Formatter;
use serde::export::fmt::Error;
use crate::oauth::token::Token;

pub struct AccessToken;

impl token::Token for AccessToken {
    fn token(&self) -> String {
        "access".to_string()
    }
}

impl token::AccessToken for AccessToken {
    fn scope(&self) -> Scope {
        Scope::from("profile")
    }

    fn expires_in(&self) -> u16 {
        3600
    }
}

impl ToString for AccessToken {
    fn to_string(&self) -> String {
        self.token()
    }
}

pub struct RefreshToken;

impl token::Token for RefreshToken {
    fn token(&self) -> String {
        "access".to_string()
    }
}

impl token::RefreshToken for RefreshToken {
    fn scope(&self) -> Scope {
        Scope::from("profile")
    }

    fn expires_in(&self) -> u16 {
        3600
    }
}

impl ToString for RefreshToken {
    fn to_string(&self) -> String {
        self.token()
    }
}