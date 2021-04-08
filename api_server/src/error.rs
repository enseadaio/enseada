use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

use api::tonic::transport::Error as TonicError;
use config::ConfigError;

#[derive(Debug)]
pub enum Error {
    ConfigError(ConfigError),
    HttpError,
    GrpcError(TonicError),
    InitError(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::ConfigError(err) => write!(f, "Configuration error: {}", err),
            Error::HttpError => write!(f, "HTTP Server error"),
            Error::GrpcError(err) => write!(f, "GRPC Server error: {}", err),
            Error::InitError(reason) => write!(f, "Initialization failed: {}", reason),
        }
    }
}

impl StdError for Error {}

impl From<ConfigError> for Error {
    fn from(err: ConfigError) -> Self {
        Error::ConfigError(err)
    }
}

impl From<TonicError> for Error {
    fn from(err: TonicError) -> Self {
        Error::GrpcError(err)
    }
}
