use serde::{Serialize, Deserialize};
use actix_web::{
    error::{BlockingError, ResponseError},
    http::StatusCode,
    HttpResponse,
};
use derive_more::Display;
use std::fmt;
use serde::export::Formatter;
use crate::couchdb;
use crate::couchdb::error::Error as CouchError;

#[derive(Debug)]
pub struct Error {
    message: String,
    source: Option<Box<dyn std::error::Error>>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| &**e as _)
    }
}

impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Error::from(message.to_string())
    }
}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Error { message, source: None }
    }
}

impl From<couchdb::error::Error> for Error {
    fn from(err: couchdb::error::Error) -> Self {
        Error { message: err.to_string(), source: Some(Box::new(err)) }
    }
}
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error { message: err.to_string(), source: Some(Box::new(err))}
    }
}

#[derive(Debug, Display, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ApiError {
    BadRequest(String),
    BlockingError(String),
    Conflict(String),
    InternalServerError(String),
    NotFound(String),
    #[display(fmt = "")]
    ValidationError(Vec<String>),
    Unauthorized(String),
    ServiceUnavailable(String),
}

/// User-friendly error messages
#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorResponse {
    errors: Vec<String>,
}

/// Automatically convert ApiErrors to external Response Errors
impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::BadRequest(error) => {
                HttpResponse::BadRequest().json::<ErrorResponse>(error.into())
            }
            ApiError::Conflict(error) => {
                HttpResponse::Conflict().json::<ErrorResponse>(error.into())
            }
            ApiError::NotFound(message) => {
                HttpResponse::NotFound().json::<ErrorResponse>(message.into())
            }
            ApiError::ValidationError(errors) => {
                HttpResponse::UnprocessableEntity().json::<ErrorResponse>(errors.to_vec().into())
            }
            ApiError::Unauthorized(error) => {
                HttpResponse::Unauthorized().json::<ErrorResponse>(error.into())
            }
            ApiError::ServiceUnavailable(error) => {
                HttpResponse::ServiceUnavailable().json::<ErrorResponse>(error.into())
            }
            _ => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

/// Utility to make transforming a string reference into an ErrorResponse
impl From<&String> for ErrorResponse {
    fn from(error: &String) -> Self {
        ErrorResponse {
            errors: vec![error.into()],
        }
    }
}

/// Utility to make transforming a vector of strings into an ErrorResponse
impl From<Vec<String>> for ErrorResponse {
    fn from(errors: Vec<String>) -> Self {
        ErrorResponse { errors }
    }
}

/// Convert Thread BlockingErrors to ApiErrors
impl From<BlockingError<ApiError>> for ApiError {
    fn from(error: BlockingError<ApiError>) -> ApiError {
        match error {
            BlockingError::Error(api_error) => api_error,
            BlockingError::Canceled => ApiError::BlockingError("Thread blocking error".into()),
        }
    }
}

impl From<Error> for ApiError {
    fn from(err: Error) -> Self {
        ApiError::InternalServerError(err.to_string())
    }
}

impl From<CouchError> for ApiError {
    fn from(err: CouchError) -> Self {
        match err {
            CouchError::Generic(err) => ApiError::InternalServerError(err),
            CouchError::NotFound(err) => ApiError::NotFound(err),
            CouchError::Conflict(err) => ApiError::Conflict(err),
        }
    }
}
