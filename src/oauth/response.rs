use std::fmt::{self, Debug, Formatter};

use serde::{Serialize};


use crate::oauth::{Result, Scope};
use crate::oauth::request::AuthorizationRequest;

#[derive(Debug, Serialize)]
pub struct AuthorizationResponse {
        code: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        state: Option<String>,
}

impl AuthorizationResponse {
        pub fn from_req(req: &AuthorizationRequest) -> Result<AuthorizationResponse> {
                // TODO: implement

                Ok(AuthorizationResponse {
                        code: String::from("code"),
                        state: req.state.as_ref().map(String::clone),
                })
        }
}

#[derive(Debug, Default, Serialize)]
pub struct TokenResponse {
        pub access_token: String,
        pub token_type: TokenType,
        pub expires_in: u16,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub refresh_token: Option<String>,
        pub scope: Scope,
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