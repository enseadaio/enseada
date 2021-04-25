use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::time::Duration;

#[derive(Debug)]
pub struct ReconciliationError<E: Error> {
    retry_in: Option<Duration>,
    cause: E,
}

impl<E: Error> ReconciliationError<E> {
    pub fn wrap(cause: E) -> Self {
        Self {
            retry_in: None,
            cause,
        }
    }

    pub fn wrap_and_retry(cause: E) -> Self {
        Self {
            retry_in: Some(Duration::from_secs(30)),
            cause,
        }
    }

    pub fn wrap_with_retry(cause: E, retry_in: Duration) -> Self {
        Self {
            retry_in: Some(retry_in),
            cause,
        }
    }

    pub fn retry_in(&self) -> Option<Duration> {
        self.retry_in
    }

    pub fn cause(&self) -> &E {
        &self.cause
    }

    pub fn into_cause(self) -> E {
        self.cause
    }
}

impl<E: Error> From<E> for ReconciliationError<E> {
    fn from(err: E) -> Self {
        Self::wrap_and_retry(err)
    }
}

impl<E: Error> Display for ReconciliationError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.cause, f)
    }
}

impl<E: Error + 'static> Error for ReconciliationError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.cause)
    }
}

#[derive(Debug)]
pub enum ControllerError {
    InitError(String),
    DatabaseError(couchdb::error::Error),
    Internal(String),
}

impl Display for ControllerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ControllerError::InitError(msg) => Display::fmt(msg, f),
            ControllerError::Internal(msg) => Display::fmt(msg, f),
            _ => Display::fmt(self.source().unwrap(), f)
        }
    }
}

impl Error for ControllerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ControllerError::DatabaseError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<couchdb::error::Error> for ControllerError {
    fn from(err: couchdb::error::Error) -> Self {
        Self::DatabaseError(err)
    }
}

impl From<String> for ControllerError {
    fn from(err: String) -> Self {
        Self::Internal(err)
    }
}

impl From<&str> for ControllerError {
    fn from(err: &str) -> Self {
        Self::from(err.to_string())
    }
}
