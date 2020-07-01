pub use routes::mount;

mod digest;
pub mod entity;
mod error;
mod header;
mod manifest;
mod mime;
mod routes;
pub mod service;
mod storage;

pub type Result<T> = std::result::Result<T, error::Error>;
