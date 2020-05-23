pub use repo::add_repo_service;
pub use routes::routes;

use crate::containers::error::Error;

pub mod digest;
pub mod error;
pub mod handler;
pub mod header;
pub mod manifest;
pub mod mime;
pub mod name;
mod repo;
mod routes;
mod storage;
mod upload;

pub type Result<T> = std::result::Result<T, Error>;