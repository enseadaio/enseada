use actix_web::{Error as HttpError, HttpResponse, ResponseError};
use actix_web::error::BlockingError;
use actix_web::http::StatusCode;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use url::ParseError;

use crate::couchdb::error::Error as CouchError;
use crate::error::Error;
use crate::oauth::error::{Error as OAuthError, ErrorKind};

#[derive(Debug, Display, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ApiError {
    BadRequest(String),
    BlockingError(String),
    Conflict(String),
    Forbidden(String),
    InternalServerError(String),
    NotFound(String),
    #[display(fmt = "")]
    ValidationError(Vec<String>),
    Unauthorized(String),
    ServiceUnavailable(String),
}

/// User-friendly error expiredmessages
#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorResponse {
    errors: Vec<String>,
}

impl ApiError {
    pub fn unauthorized() -> Self {
        ApiError::Unauthorized("unauthorized".to_string())
    }
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
            ApiError::Forbidden(error) => {
                HttpResponse::Forbidden().json::<ErrorResponse>(error.into())
            }
            ApiError::NotFound(error) => {
                HttpResponse::NotFound().json::<ErrorResponse>(error.into())
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
            _ => HttpResponse::InternalServerError().finish(),
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

impl From<HttpError> for ApiError {
    fn from(err: HttpError) -> Self {
        ApiError::InternalServerError(err.to_string())
    }
}

impl From<url::ParseError> for ApiError {
    fn from(err: ParseError) -> Self {
        ApiError::BadRequest(err.to_string())
    }
}

impl From<OAuthError> for ApiError {
    fn from(err: OAuthError) -> Self {
        let message = err.description().to_string();
        match err.kind() {
            ErrorKind::AccessDenied => ApiError::Forbidden(message),
            ErrorKind::InvalidClient => ApiError::Unauthorized(message),
            ErrorKind::ServerError | ErrorKind::Unknown => ApiError::InternalServerError(message),
            ErrorKind::TemporarilyUnavailable => ApiError::ServiceUnavailable(message),
            _ => ApiError::BadRequest(message),
        }
    }
}