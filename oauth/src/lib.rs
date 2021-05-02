pub use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::Error;

pub mod client;
pub mod code;
pub mod error;
pub mod handler;
pub mod request;
pub mod response;
pub mod scope;
pub mod session;
pub mod token;

pub type Result<T> = std::result::Result<T, Error>;

pub trait Expirable {
    fn expiration(&self) -> DateTime<Utc>;
    fn expires_in(&self) -> i64;
    fn is_expired(&self) -> bool;
}
