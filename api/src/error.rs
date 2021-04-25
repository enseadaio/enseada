use serde::Serialize;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use http::StatusCode;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Code {
    InvalidBody,
    NotFound,
    Unknown
}

impl Code {
    pub fn to_status(&self) -> StatusCode {
        match self {
            Code::InvalidBody => StatusCode::BAD_REQUEST,
            Code::NotFound => StatusCode::NOT_FOUND,
            Code::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl Default for Code {
    fn default() -> Self {
        Self::Unknown
    }
}

impl Display for Code {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Code::InvalidBody => Display::fmt("invalid_body", f),
            Code::NotFound => Display::fmt("not_found", f),
            Code::Unknown => Display::fmt("unknown", f),
        }
    }
}

impl Into<http::StatusCode> for Code {
    fn into(self) -> StatusCode {
        self.to_status()
    }
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub code: Code,
    pub message: String,
    pub metadata: Option<HashMap<String, String>>,
}
