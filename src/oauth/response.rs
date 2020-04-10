use std::fmt::{self, Debug, Formatter};

use serde::Serialize;



use crate::oauth::scope::Scope;

use crate::oauth::code::AuthorizationCode;
use std::collections::HashMap;
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct AuthorizationResponse {
    code: AuthorizationCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<String>,
}

impl AuthorizationResponse {
    pub fn new(code: AuthorizationCode, state: Option<String>) -> AuthorizationResponse {
        AuthorizationResponse {
            code,
            state,
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: TokenType,
    pub expires_in: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    pub scope: Scope,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Bearer
}

impl Debug for TokenType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(s) => write!(f, "{}", s),
            Err(_) => Err(fmt::Error),
        }
    }
}

impl Default for TokenType {
    fn default() -> Self {
        Self::Bearer
    }
}