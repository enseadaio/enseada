use serde::Deserialize;

use crate::http::error::ApiError;

pub mod error;
pub mod extractor;
pub mod guard;
pub mod header;
pub mod middleware;
pub mod responses;

pub type ApiResult<T> = Result<T, ApiError>;

fn default_limit() -> usize {
    20
}

fn default_offset() -> usize {
    0
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default = "default_offset")]
    offset: usize,
}

impl PaginationQuery {
    pub fn limit(&self) -> usize {
        self.limit
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}
