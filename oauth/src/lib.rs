pub use request::*;

pub mod error;
pub mod request;
pub mod handler;
pub mod scope;
pub mod session;
pub mod token;

/// Represent HTTP basic authentication as (client_id, client_secret)
#[derive(Debug)]
pub struct BasicAuth(String, Option<String>);

impl BasicAuth {
    pub fn new(username: String, password: Option<String>) -> Self {
        BasicAuth(username, password)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Expirable {
    fn expiration(&self) -> DateTime<Utc>;
    fn expires_in(&self) -> i64;
    fn is_expired(&self) -> bool;
}
