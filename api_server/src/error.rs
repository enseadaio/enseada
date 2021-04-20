use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

use config::ConfigError;
use api::error::{Code, ErrorResponse};
use warp::reject::{Reject, custom};
use warp::{Rejection, Reply};
use std::convert::Infallible;
use warp::reply::{with_status, json};

#[derive(Debug)]
pub enum Error {
    ConfigError(ConfigError),
    ApiError {
        code: Code,
        message: String,
    },
    InitError(String),
}

impl Error {
    pub fn internal<M: ToString>(message: M) -> Self {
        Self::ApiError {
            code: Code::Unknown,
            message: message.to_string(),
        }
    }

    pub fn not_found<M: ToString>(message: M) -> Self {
        Self::ApiError {
            code: Code::NotFound,
            message: message.to_string(),
        }
    }
}

impl Reject for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::ConfigError(err) => write!(f, "Configuration error: {}", err),
            Error::InitError(reason) => write!(f, "Initialization failed: {}", reason),
            Error::ApiError{ code, message} => write!(f, "API error: {} {}", code, message)
        }
    }
}

impl StdError for Error {}

impl From<ConfigError> for Error {
    fn from(err: ConfigError) -> Self {
        Error::ConfigError(err)
    }
}

impl From<couchdb::error::Error> for Error {
    fn from(err: couchdb::error::Error) -> Self {
        match err.status() {
            // TODO: properly map couch errors
            _ => Self::ApiError { code: Code::Unknown, message: err.to_string() },
        }
    }
}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;
    let metadata = None;

    if err.is_not_found() {
        code = Code::NotFound;
        message = "not found".to_string();
    } else if let Some(Error::ApiError { code: err_code, message: err_message }) = err.find::<Error>() {
        code = err_code.clone();
        message = err_message.clone();
    } else {
        code = Code::Unknown;
        message = "internal server error".to_string();
    }

    let status = code.to_status();
    let json = json(&ErrorResponse {
        code,
        message,
        metadata,
    });

    Ok(with_status(json, status))
}
