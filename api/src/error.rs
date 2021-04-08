// Based on tonic::transport::Error since it's crate-scoped and we can't use it directly.

use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter, Debug};
use http::uri::InvalidUri;

type Source = Box<dyn StdError + Send + Sync + 'static>;

pub struct Error {
    kind: Kind,
    source: Option<Source>,
}

#[derive(Debug)]
pub(crate) enum Kind {
    Tonic,
    Uri,
}

impl Error {
    pub(crate) fn new(kind: Kind) -> Self {
        Self {
            kind,
            source: None,
        }
    }

    pub(crate) fn with_source(mut self, source: impl Into<Source>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn description(&self) -> &str {
        match self.kind {
            Kind::Tonic => "gRPC error",
            Kind::Uri => "URI parsing error",
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_tuple("api::error::Error");

        f.field(&self.kind);

        if let Some(source) = &self.source {
            f.field(source);
        }

        f.finish()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(source) = &self.source {
            write!(f, "{}: {}", self.description(), source)
        } else {
            f.write_str(self.description())
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_ref()
            .map(|source| &**source as &(dyn StdError + 'static))
    }
}

impl From<tonic::transport::Error> for Error {
    fn from(err: tonic::transport::Error) -> Self {
        Self::new(Kind::Tonic).with_source(err)
    }
}

impl From<InvalidUri> for Error {
    fn from(err: InvalidUri) -> Self {
        Self::new(Kind::Uri).with_source(err)
    }
}
