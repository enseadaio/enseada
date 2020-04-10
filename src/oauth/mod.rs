use async_trait::async_trait;

use crate::oauth::error::Error;
use crate::oauth::session::Session;
use chrono::{DateTime, Utc};

pub mod client;
pub mod code;
pub mod error;
pub mod handler;
pub mod request;
pub mod response;
pub mod session;
pub mod scope;
pub mod storage;
pub mod token;
pub mod persistence;


pub type Result<T> = std::result::Result<T, Error>;

#[async_trait]
pub trait RequestHandler<T, R> {
    async fn validate(&self, req: &T) -> Result<()>;
    async fn handle(&self, req: &T, session: &mut Session) -> Result<R>;
}

pub trait Expirable {
    fn expiration(&self) -> &DateTime<Utc>;
    fn expires_in(&self) -> i64;
    fn is_expired(&self) -> bool;
}