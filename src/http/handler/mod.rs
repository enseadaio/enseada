use crate::http::error::ApiError;

pub mod api_docs;
pub mod health;
pub mod oauth;
pub mod rbac;
pub mod ui;
pub mod user;

pub type ApiResult<T> = Result<T, ApiError>;