use std::fmt::{self, Debug, Display, Formatter};

use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Error {
    error: ErrorKind,
    error_description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_uri: Option<String>,
}

impl Error {
    pub fn new(kind: ErrorKind, description: String) -> Error {
        Error {
            error: kind,
            error_description: description,
            error_uri: None,
        }
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.error
    }

    pub fn set_error_uri(&mut self, url: url::Url) -> &mut Self {
        self.error_uri = Some(url.to_string());
        self
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error, self.error_description)
    }
}

impl std::error::Error for Error {}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Error::new(ErrorKind::ServerError, message)
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        match self.kind() {
            ErrorKind::AccessDenied => HttpResponse::Forbidden(),
            ErrorKind::InvalidClient => HttpResponse::Unauthorized(),
            ErrorKind::ServerError | ErrorKind::Unknown => HttpResponse::InternalServerError(),
            ErrorKind::TemporarilyUnavailable => HttpResponse::ServiceUnavailable(),
            _ => HttpResponse::BadRequest(),
        }.json(self)
    }
}

#[derive(Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    AccessDenied,
    InvalidClient,
    InvalidGrant,
    InvalidRedirectUri,
    InvalidRequest,
    InvalidScope,
    ServerError,
    TemporarilyUnavailable,
    UnauthorizedClient,
    Unknown,
    UnsupportedGrantType,
    UnsupportedResponseType,
}

impl Debug for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(s) => write!(f, "{}", s),
            Err(_) => Err(fmt::Error),
        }
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}