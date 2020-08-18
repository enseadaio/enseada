use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};

use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

use crate::code::AuthorizationCode;
use crate::scope::Scope;
use crate::token::{Token, TokenTypeHint};

#[derive(Debug, Serialize)]
pub struct AuthorizationResponse {
    code: AuthorizationCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<String>,
}

impl AuthorizationResponse {
    pub fn new(code: AuthorizationCode, state: Option<String>) -> AuthorizationResponse {
        AuthorizationResponse { code, state }
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
    Bearer,
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

#[derive(Debug, Serialize)]
pub struct IntrospectionResponse {
    pub active: bool,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub introspection_data: Option<ActiveIntrospectionResponse>,
}

#[derive(Debug, Serialize)]
pub struct ActiveIntrospectionResponse {
    pub scope: Scope,
    pub client_id: String,
    pub username: Option<String>,
    pub token_type: TokenTypeHint,
    #[serde(with = "ts_seconds")]
    pub exp: DateTime<Utc>,
}

impl IntrospectionResponse {
    pub fn from_token<T: Token>(token: &T) -> Self {
        if token.is_expired() {
            IntrospectionResponse::inactive()
        } else {
            IntrospectionResponse::active(token)
        }
    }

    pub fn inactive() -> Self {
        IntrospectionResponse {
            active: false,
            introspection_data: None,
        }
    }

    pub fn active<T: Token>(token: &T) -> Self {
        let session = token.session();
        IntrospectionResponse {
            active: true,
            introspection_data: Some(ActiveIntrospectionResponse {
                scope: session.scope().clone(),
                client_id: session.client_id().clone(),
                username: session.user_id().clone(),
                token_type: token.type_hint(),
                exp: *token.expiration(),
            }),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RevocationResponse {
    ok: bool,
}

impl RevocationResponse {
    pub fn ok() -> Self {
        RevocationResponse { ok: true }
    }
}
