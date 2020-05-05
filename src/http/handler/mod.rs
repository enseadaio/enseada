use serde::Deserialize;

use crate::http::error::ApiError;
use crate::pagination::Cursor;

pub mod api_docs;
pub mod health;
pub mod oauth;
pub mod rbac;
pub mod ui;
pub mod user;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    limit: Option<usize>,
    cursor: Option<String>,
}

impl PaginationQuery {
    pub fn limit(&self) -> usize {
        self.limit.unwrap_or(20)
    }

    pub fn cursor(&self) -> Option<&String> {
        self.cursor.as_ref()
    }
}