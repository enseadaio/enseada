use std::char::ToLowercase;

use actix_web::{HttpRequest, HttpResponse};
use actix_web::http::HeaderValue;
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

pub async fn home(req: HttpRequest) -> HttpResponse {
    let accept = req.headers().get(http::header::ACCEPT)
        .and_then(|accept| accept.to_str().ok())
        .map(str::to_lowercase)
        .filter(|accept| (*accept).contains("html"));
    let redirect = match accept {
        Some(_) => "/ui",
        None => "/health",
    };
    HttpResponse::SeeOther()
        .header(http::header::LOCATION, redirect)
        .finish()
}