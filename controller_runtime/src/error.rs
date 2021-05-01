use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::time::Duration;
use http::StatusCode;
use std::any::Any;
use std::convert::TryInto;

#[derive(Debug)]
pub struct ReconciliationError<E: Error> {
    retry_in: Option<Duration>,
    cause: E,
}

impl<E: 'static + Error> ReconciliationError<E> {
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

    pub fn cause_as<Err: 'static + Error>(&self) -> Option<&Err> {
        let err = (&self.cause) as &(dyn Any);
        err.downcast_ref::<Err>()
    }

    pub fn into_cause(self) -> E {
        self.cause
    }
}

impl<E: 'static + Error> From<E> for ReconciliationError<E> {
    fn from(err: E) -> Self {
        Self::wrap_and_retry(err)
    }
}

impl<E: Error> Display for ReconciliationError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.cause, f)
    }
}

impl<E: 'static + Error> Error for ReconciliationError<E> {
    fn source(&self) -> Option<&(dyn 'static + Error)> {
        Some(&self.cause)
    }
}

impl<E: 'static + Error> TryInto<ControllerError> for ReconciliationError<E> {
    type Error = ();

    fn try_into(self) -> Result<ControllerError, Self::Error> {
        let err = (&self.cause) as &(dyn Any);
        match err.downcast_ref::<ControllerError>() {
            Some(err) => Ok(err.clone()),
            None => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ControllerError {
    InitError(String),
    RevisionConflict,
    DatabaseError(couchdb::error::Error),
    Internal(String),
}

impl Display for ControllerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ControllerError::InitError(msg) => Display::fmt(msg, f),
            ControllerError::Internal(msg) => Display::fmt(msg, f),
            ControllerError::RevisionConflict => Display::fmt("Revision conflict detected", f),
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
        match err.status() {
            StatusCode::CONFLICT => Self::RevisionConflict,
            _ => Self::DatabaseError(err),
        }

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
