use crate::oauth::Scope;
use std::fmt::{self, Display, Formatter};

pub trait Token: ToString {
    fn token(&self) -> String;
}

pub trait AccessToken: Token {
    fn scope(&self) -> Scope;
    fn expires_in(&self) -> u16;
}


pub trait RefreshToken: Token {
    fn scope(&self) -> Scope;
    fn expires_in(&self) -> u16;
}
