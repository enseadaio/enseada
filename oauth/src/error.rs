use std::fmt::{self, Debug, Display, Formatter};

use http::StatusCode;
use serde::Serialize;

use enseada::couchdb;

#[derive(Serialize, Debug)]
pub struct Error {
    error: ErrorKind,
    error_description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_uri: Option<String>,
    state: Option<String>,
}

impl Error {
    pub fn new<D: ToString>(kind: ErrorKind, description: D) -> Self {
        Error {
            error: kind,
            error_description: description.to_string(),
            error_uri: None,
            state: None,
        }
    }

    pub fn with_state<D: ToString, S: ToString>(
        kind: ErrorKind,
        description: D,
        state: Option<S>,
    ) -> Self {
        let mut err = Self::new(kind, description);
        if state.is_some() {
            err.set_state(state);
        }
        err
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.error
    }

    pub fn description(&self) -> &str {
        &self.error_description
    }

    pub fn set_state<S: ToString>(&mut self, state: Option<S>) -> &mut Self {
        self.state = state.map(|s| s.to_string());
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

impl From<couchdb::error::Error> for Error {
    fn from(err: couchdb::error::Error) -> Self {
        match err.status() {
            StatusCode::NOT_FOUND => Self::new(ErrorKind::AccessDenied, err),
            _ => Self::from(err.to_string()),
        }
    }
}

#[derive(Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    AccessDenied,
    AuthenticationFailed,
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
