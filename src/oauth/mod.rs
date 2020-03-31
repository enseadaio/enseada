


use crate::oauth::error::{Error};

pub mod client;
pub mod code;
pub mod error;
pub mod handler;
pub mod request;
pub mod response;
pub mod storage;
pub mod token;
pub mod persistence;

mod scope;
pub use scope::*;

pub type Result<T> = std::result::Result<T, Error>;

pub trait RequestHandler<T, R> {
    fn validate(&self, req: &T) -> Result<()>;
    fn handle(&self, req: &T) -> Result<R>;
}
