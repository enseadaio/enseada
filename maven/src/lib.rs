use enseada::error::Error;
pub use maven_version::*;

pub mod entity;
pub mod file;
pub mod service;
mod storage;

pub type Result<T> = std::result::Result<T, Error>;
