use serde::Serialize;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use http::StatusCode;

#[derive(Copy, Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Code {
    InvalidRequest,
    NotFound,
    InitializationFailed,
    Unknown,
    UnsupportedMediaType,
    InvalidHeader,
}

impl Code {
    pub fn to_status(&self) -> StatusCode {
        match self {
            Code::InvalidRequest => StatusCode::BAD_REQUEST,
            Code::NotFound => StatusCode::NOT_FOUND,
            Code::InitializationFailed => StatusCode::INTERNAL_SERVER_ERROR,
            Code::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            Code::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Code::InvalidHeader => StatusCode::BAD_REQUEST,
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
            Code::InvalidRequest => Display::fmt("invalid_request", f),
            Code::NotFound => Display::fmt("not_found", f),
            Code::InitializationFailed => Display::fmt("initialization_failed", f),
            Code::Unknown => Display::fmt("unknown", f),
            Code::UnsupportedMediaType => Display::fmt("unsupported_media_type", f),
            Code::InvalidHeader => Display::fmt("invalid_header", f),
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
