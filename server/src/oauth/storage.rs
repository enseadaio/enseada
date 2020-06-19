use async_trait::async_trait;
use enseada::pagination::{Cursor, Page};

use crate::oauth::client::Client;
use crate::oauth::code::AuthorizationCode;
use crate::oauth::token::Token;
use crate::oauth::Result;

#[async_trait]
pub trait ClientStorage: Send + Sync {
    async fn list_clients(&self, limit: usize, cursor: Option<&Cursor>) -> Result<Page<Client>>;
    async fn get_client(&self, id: &str) -> Option<Client>;
    async fn save_client(&self, client: Client) -> Result<Client>;
    async fn delete_client(&self, client: &Client) -> Result<()>;
}

#[async_trait]
pub trait TokenStorage<T: Token>: Send + Sync {
    async fn get_token(&self, sig: &str) -> Option<T>;
    async fn store_token(&self, sig: &str, token: T) -> Result<T>;
    async fn revoke_token(&self, sig: &str) -> Result<()>;
}

#[async_trait]
pub trait AuthorizationCodeStorage: Send + Sync {
    async fn get_code(&self, sig: &str) -> Option<AuthorizationCode>;
    async fn store_code(&self, sig: &str, code: AuthorizationCode) -> Result<AuthorizationCode>;
    async fn revoke_code(&self, sig: &str) -> Result<()>;
}
