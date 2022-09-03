use hyper::body::Bytes;
use hyper::http;
use hyper::http::response::Parts;
use serde::Deserialize;

use std::fmt::Debug;

use crate::response::CouchErrorBody;

use crate::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FutonError {
    #[error("http error: {0}")]
    Http(#[from] http::Error),
    #[error("hyper error: {0}")]
    Hyper(#[from] hyper::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("query string error: {0}")]
    QueryString(#[from] serde_qs::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("couchdb error: {0}")]
    Couch(CouchError),
    #[error("document conflict: {0}")]
    Conflict(CouchError),
    #[error("an unknown error has occurred")]
    Unknown,
}

impl From<CouchError> for FutonError {
    fn from(err: CouchError) -> Self {
        match err {
            CouchError::Hyper(hyper) => Self::Hyper(hyper),
            CouchError::Json(json) => Self::Json(json),
            CouchError::Conflict(_) => Self::Conflict(err),
            couch => Self::Couch(couch),
        }
    }
}

#[derive(Debug, Error)]
pub enum CouchError {
    #[error("hyper error: {0}")]
    Hyper(#[from] hyper::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Couch(String),
    #[error("{0}")]
    Conflict(String),
    #[error("internal server error")]
    Internal,
}

impl From<(Parts, Bytes)> for CouchError {
    fn from((parts, body): (Parts, Bytes)) -> Self {
        let msg = match serde_json::from_slice::<CouchErrorBody>(&body) {
            Ok(err) => format!("{}: {}", err.error, err.reason),
            _ => format!("couch error: {body:?}"),
        };

        match parts.status {
            StatusCode::CONFLICT => Self::Conflict(msg),
            _ => Self::Couch(msg),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CouchErrorResponse {}
