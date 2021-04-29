use std::fmt::{self, Debug, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::request::pkce::PkceRequest;
use crate::scope::Scope;

#[derive(Clone, Debug, Deserialize)]
pub struct AuthorizationRequest {
    pub response_type: ResponseType,
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: Scope,
    pub state: Option<String>,
    #[serde(flatten)]
    pub pkce: Option<PkceRequest>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseType {
    Code,
}

impl Debug for ResponseType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(s) => Debug::fmt(&s, f),
            Err(_) => Err(fmt::Error),
        }
    }
}

impl From<String> for ResponseType {
    fn from(typ: String) -> Self {
        match typ.as_str() {
            _ => ResponseType::Code,
        }
    }
}

impl Display for ResponseType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            ResponseType::Code => "code",
        };
        Display::fmt(&s, f)
    }
}
