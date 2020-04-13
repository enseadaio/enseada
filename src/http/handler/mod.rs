use crate::error::ApiError;

pub mod health;
pub mod oauth;
pub mod ui;
pub mod user;

pub type ApiResult<T> = Result<T, ApiError>;