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

/// User-friendly error messages
#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorResponse {
    error: String,
    reasons: Vec<String>,
}

impl ApiError {
    pub fn unauthorized() -> Self {
        ApiError::Unauthorized("unauthorized".to_string())
    }
}

/// Automatically convert ApiErrors to external Response Errors
impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::BadRequest(error) => {
                HttpResponse::BadRequest().json::<ErrorResponse>(ErrorResponse::new(self.status_code(), vec![error.clone()]))
            }
            ApiError::Conflict(error) => {
                HttpResponse::Conflict().json::<ErrorResponse>(ErrorResponse::new(self.status_code(), vec![error.clone()]))
            }
            ApiError::Forbidden(error) => {
                HttpResponse::Forbidden().json::<ErrorResponse>(ErrorResponse::new(self.status_code(), vec![error.clone()]))
            }
            ApiError::NotFound(error) => {
                HttpResponse::NotFound().json::<ErrorResponse>(ErrorResponse::new(self.status_code(), vec![error.clone()]))
            }
            ApiError::ValidationError(errors) => {
                HttpResponse::UnprocessableEntity().json::<ErrorResponse>(ErrorResponse::new(self.status_code(), errors.clone()))
            }
            ApiError::Unauthorized(error) => {
                HttpResponse::Unauthorized().json::<ErrorResponse>(ErrorResponse::new(self.status_code(), vec![error.clone()]))
            }
            ApiError::ServiceUnavailable(error) => {
                HttpResponse::ServiceUnavailable().json::<ErrorResponse>(ErrorResponse::new(self.status_code(), vec![error.clone()]))
            }
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}

impl ErrorResponse {
    pub fn new(status: StatusCode, reasons: Vec<String>) -> ErrorResponse {
        ErrorResponse {
            error: status.canonical_reason().unwrap_or_else(|| "Internal Server Error").to_string(),
            reasons,
        }
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
        let message = err.to_string();
        let status = err.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        match status {
            StatusCode::CONFLICT => ApiError::Conflict(message),
            StatusCode::NOT_FOUND => ApiError::NotFound(message),
            _ => ApiError::InternalServerError(message),
        }
    }
}

impl From<CouchError> for ApiError {
    fn from(err: CouchError) -> Self {
        let message = err.to_string();
        match err.status() {
            StatusCode::CONFLICT => ApiError::Conflict(message),
            StatusCode::NOT_FOUND => ApiError::NotFound(message),
            _ => ApiError::InternalServerError(message),
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