use async_trait::async_trait;

use crate::BasicAuth;
use crate::Result;
use crate::session::Session;
use crate::token::Token;

#[async_trait]
pub trait RequestHandler<T, R> {
    async fn validate(&self, req: &T, client_auth: Option<&BasicAuth>) -> Result<Client>;
    async fn handle(&self, req: &T, session: &mut Session) -> Result<R>;
}

#[async_trait]
pub trait TokenIntrospectionHandler<T: Token> {
    async fn get_token(&self, token: &str) -> Result<T>;
    async fn revoke_token(&self, token: &str) -> Result<()>;
}
