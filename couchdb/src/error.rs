use std::fmt::Display;

use reqwest::StatusCode;
use serde::export::Formatter;

use crate::migrator;

#[derive(Debug, PartialEq, Eq)]
pub struct Error {
    message: String,
    status: StatusCode,
}

impl Error {
    pub fn not_found(message: String) -> Self {
        Error {
            message,
            status: StatusCode::NOT_FOUND,
        }
    }

    pub fn status(&self) -> StatusCode {
        self.status
    }

    pub(super) fn map_message(err: reqwest::Error, message: &str) -> Error {
        let mut mapped = Self::from(err);
        mapped.message = message.to_string();
        mapped
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        let message = err.to_string();
        let status = err.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        Error { status, message }
    }
}

impl From<migrator::MigrationError> for Error {
    fn from(err: migrator::MigrationError) -> Self {
        Error {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: err.to_string(),
        }
    }
}
