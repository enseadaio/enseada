use std::fmt::{self, Display, Formatter};
use serde::{Serialize, Deserialize, Deserializer, Serializer};

use crate::oauth::error::{Error, ErrorKind};

pub mod request;
pub mod response;
pub mod client;
pub mod handler;
pub mod error;
pub mod token;

mod scope;
pub use scope::*;

pub type Result<T> = std::result::Result<T, Error>;

pub trait RequestHandler<T, R> {
    fn validate(&self, req: &T) -> Result<()>;
    fn handle(&self, req: &T) -> Result<R>;
}
