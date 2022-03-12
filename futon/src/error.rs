use hyper::{http, Response};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FutonError {
    #[error("http error: {0}")]
    Http(#[from] http::Error),
    #[error("hyper error: {0}")]
    Hyper(#[from] hyper::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("couchdb error: {0}")]
    Couch(#[from] CouchError),
    #[error("an unknown error has occurred")]
    Unknown,
}

#[derive(Debug, Error)]
pub enum CouchError {
    #[error("internal server error")]
    Internal,
}

impl<B> From<Response<B>> for CouchError {
    fn from(res: Response<B>) -> Self {
        match res.status() {
            _ => Self::Internal,
        }
    }
}
