use crate::oauth::Result;
use crate::oauth::client::Client;
use crate::oauth::token::Token;
use crate::oauth::code::AuthorizationCode;

pub trait ClientStorage<C: Client> {
    fn get_client(&self, id: &str) -> Option<C>;
}

pub trait TokenStorage<T: Token> {
    fn get_token(&self, sig: &str) -> Option<T>;
    fn store_token(&self, sig: &str, token: T) -> Result<T>;
    fn revoke_token(&self, sig: &str) -> Result<()>;
}

pub trait AuthorizationCodeStorage<C: AuthorizationCode> {
    fn get_code(&self, sig: &str) -> Option<C>;
    fn store_code(&self, sig: &str, code: C) -> Result<C>;
    fn revoke_code(&self, sig: &str) -> Result<()>;
}