use std::fmt::{self, Debug, Formatter};

use serde::{Deserialize, Serialize};

use crate::oauth::scope::Scope;

#[derive(Debug, Deserialize)]
pub struct AuthorizationRequest {
    pub response_type: ResponseType,
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: Scope,
    pub state: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseType {
    Code
}

impl Debug for ResponseType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(s) => s.fmt(f),
            Err(_) => Err(fmt::Error),
        }
    }
}

impl From<String> for ResponseType {
    fn from(typ: String) -> Self {
        match typ.as_str() {
            "code" | _ => ResponseType::Code,
        }
    }
}

impl ToString for ResponseType {
    fn to_string(&self) -> String {
        match self {
            ResponseType::Code => "code".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "grant_type", rename_all = "snake_case")]
pub enum TokenRequest {
    AuthorizationCode {
        code: String,
        redirect_uri: String,
        client_id: Option<String>,
    },
    #[serde(other)]
    Unknown,
}

