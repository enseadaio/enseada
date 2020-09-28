use std::fmt::{self, Display, Formatter};

use crate::lexer::UnexpectedChar;

#[derive(Debug)]
pub enum Error {
    Parse(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Error::Parse(msg) = self;
        msg.fmt(f)
    }
}

impl std::error::Error for Error {}

impl From<UnexpectedChar> for Error {
    fn from(UnexpectedChar(c): UnexpectedChar) -> Self {
        Error::Parse(format!("unexpected character: {}", c))
    }
}
