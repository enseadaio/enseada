pub use routes::mount;

mod digest;
mod entity;
mod error;
mod header;
mod mime;
mod routes;
mod service;
mod storage;

pub type Result<T> = std::result::Result<T, error::Error>;
