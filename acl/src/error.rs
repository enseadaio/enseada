use std::fmt::{self, Display, Formatter};
use controller_runtime::ControllerError;

#[derive(Debug)]
pub enum Error {
    Controller(ControllerError),
    Denied(String),
}

impl Error {
    pub fn denied<M: ToString>(message: M) -> Self {
        Self::Denied(message.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Controller(err) => write!(f, "Controller error: {}", err),
            Self::Denied(msg) => msg.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<ControllerError> for Error {
    fn from(err: ControllerError) -> Self {
        Self::Controller(err)
    }
}
