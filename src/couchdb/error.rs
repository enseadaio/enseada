use std::fmt::Display;

use reqwest::StatusCode;
use serde::export::Formatter;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Generic(String),
    NotFound(String),
    Conflict(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Error::Generic(s) => s,
            Error::NotFound(s) => s,
            Error::Conflict(s) => s,
        };
        s.fmt(f)
    }
}

impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        let message = err.to_string();
        if !err.is_status() {
            return Error::Generic(message)
        }

        match err.status().unwrap() {
            StatusCode::NOT_FOUND => Error::NotFound(message),
            StatusCode::CONFLICT => Error::Conflict(message),
            _ => Error::Generic(message),
        }
    }
}