use crate::http::error::ApiError;

pub mod health;
pub mod oauth;
pub mod ui;
pub mod user;
pub mod api_docs;

pub type ApiResult<T> = Result<T, ApiError>;