use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

use config::ConfigError;
use api::error::{Code, ErrorResponse};
use warp::reject::Reject;
use controller_runtime::ControllerError;
use http::StatusCode;
use mime::Mime;

#[derive(Debug)]
pub enum Error {
    ConfigError(ConfigError),
    ApiError {
        code: Code,
        message: String,
    },
    PrometheusError(prometheus::Error),
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

    pub fn unsupported_media_type(mime: Mime) -> Self {
        Self::ApiError {
            code: Code::UnsupportedMediaType,
            message: format!("media type '{}' is unsupported", mime),
        }
    }

    pub fn invalid_header<N: AsRef<str>, V: AsRef<str>>(name: N, value: Option<V>) -> Self {
        Self::ApiError {
            code: Code::UnsupportedMediaType,
            message: format!("invalid header '{}{}'", name.as_ref(), value.as_ref().map_or("".to_string(), |v| format!(": {}", v.as_ref()))),
        }
    }

    pub fn code(&self) -> Code {
        match self {
            Error::ConfigError(_) => Code::InitializationFailed,
            Error::ApiError { code, .. } => *code,
            Error::PrometheusError(_) => Code::Unknown,
            Error::InitError(_) => Code::InitializationFailed,
        }
    }

}

impl Reject for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::ConfigError(err) => write!(f, "Configuration error: {}", err),
            Error::InitError(reason) => write!(f, "Initialization failed: {}", reason),
            Error::ApiError{ code, message} => write!(f, "API error: {} {}", code, message),
            Error::PrometheusError(err) => Display::fmt(err, f),
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
            StatusCode::NOT_FOUND => Self::ApiError { code: Code::NotFound, message: err.to_string() },
            _ => Self::ApiError { code: Code::Unknown, message: err.to_string() },
        }
    }
}

impl From<ControllerError> for Error {
    fn from(err: ControllerError) -> Self {
        match err {
            ControllerError::InitError(err) => Error::InitError(err),
            ControllerError::RevisionConflict => Error::internal("Unhandled revision conflict"),
            ControllerError::DatabaseError(err) => err.into(),
            ControllerError::Internal(err) => Error::internal(err),
        }
    }
}

impl From<prometheus::Error> for Error {
    fn from(err: prometheus::Error) -> Self {
        Error::PrometheusError(err)
    }
}

impl Into<ErrorResponse> for Error {
    fn into(self) -> ErrorResponse {
        ErrorResponse {
            code: self.code(),
            message: self.to_string(),
            metadata: None
        }
    }
}
