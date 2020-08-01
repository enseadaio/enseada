pub mod digest;
pub mod entity;
pub mod error;
pub mod header;
pub mod manifest;
pub mod mime;
pub mod service;
mod storage;

pub type Result<T> = std::result::Result<T, error::Error>;
