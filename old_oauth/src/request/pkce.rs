use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use enseada::secure;

use crate::error::{Error, ErrorKind};

#[derive(Clone, Debug, Deserialize)]
pub struct PkceRequest {
    code_challenge: String,
    code_challenge_method: TransformationMethod,
}

impl PkceRequest {
    pub fn new<S: ToString>(
        code_challenge: S,
        code_challenge_method: TransformationMethod,
    ) -> Self {
        Self {
            code_challenge: code_challenge.to_string(),
            code_challenge_method,
        }
    }
    pub fn code_challenge(&self) -> &str {
        &self.code_challenge
    }

    pub fn code_challenge_method(&self) -> &TransformationMethod {
        &self.code_challenge_method
    }

    pub fn validate<S: ToString>(&self, state: Option<S>) -> Result<(), Error> {
        self.code_challenge_method.validate(state)?;
        Ok(())
    }

    pub fn verify<S: ToString>(&self, code_verifier: S) -> Result<(), Error> {
        let code_verifier = code_verifier.to_string();
        let computed_verifier = match self.code_challenge_method {
            TransformationMethod::PLAIN => code_verifier,
            TransformationMethod::S256 => secure::pkce_challenge(&code_verifier),
        };

        if computed_verifier == self.code_challenge {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::InvalidGrant, "PKCE challenge failed"))
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum TransformationMethod {
    /// plain text (no transformation)
    #[serde(rename = "plain")]
    PLAIN,
    /// SHA-256 transformation method
    S256,
}

impl TransformationMethod {
    // We keep this method in case we will disable PLAIN method in the future
    pub fn validate<S: ToString>(&self, _state: Option<S>) -> Result<(), Error> {
        match self {
            TransformationMethod::PLAIN => Ok(()),
            TransformationMethod::S256 => Ok(()),
        }
    }
}

impl Display for TransformationMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            TransformationMethod::PLAIN => "plain",
            TransformationMethod::S256 => "S256",
        };
        s.fmt(f)
    }
}
