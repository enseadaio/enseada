use std::string::FromUtf8Error;

use http::StatusCode;
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{}", reason))]
    Conflict { reason: String },
    #[snafu(display("{} '{}' not found", typ, id))]
    NotFound { typ: String, id: String },
    #[snafu(display("{}", source))]
    FromUtf8 { source: FromUtf8Error },
    #[snafu(display("{}", source))]
    Base64Decode { source: base64::DecodeError },
    #[snafu(display("{}", source))]
    Database { source: couchdb::error::Error },
    #[snafu(display("{}", message))]
    Generic { message: String },
}

// pub struct Error {
//     status: Option<StatusCode>,
//     message: String,
//     source: Option<Box<dyn std::error::Error>>,
// }

impl Error {
    pub fn not_found(typ: &str, id: &str) -> Self {
        Error::NotFound {
            typ: typ.to_string(),
            id: id.to_string(),
        }
    }

    pub fn conflict(reason: String) -> Self {
        Error::Conflict { reason }
    }

    pub fn new(message: &str) -> Self {
        Error::Generic {
            message: message.to_string(),
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Error::Conflict { .. } => StatusCode::CONFLICT,
            Error::NotFound { .. } => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Error::from(message.to_string())
    }
}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Error::Generic { message }
    }
}

impl From<couchdb::error::Error> for Error {
    fn from(err: couchdb::error::Error) -> Self {
        Error::Database { source: err }
    }
}

impl From<base64::DecodeError> for Error {
    fn from(err: base64::DecodeError) -> Self {
        Error::Base64Decode { source: err }
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Self {
        Error::FromUtf8 { source: err }
    }
}
