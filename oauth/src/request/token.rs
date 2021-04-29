use serde::Deserialize;

use crate::scope::Scope;
use crate::token::TokenTypeHint;

#[derive(Debug, Deserialize)]
#[serde(tag = "grant_type", rename_all = "snake_case")]
pub enum TokenRequest {
    AuthorizationCode(AuthorizationCodeRequest),
    RefreshToken(RefreshTokenRequest),
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct AuthorizationCodeRequest {
    code: String,
    redirect_uri: String,
    client_id: Option<String>,
    client_secret: Option<String>,
    code_verifier: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    refresh_token: String,
    scope: Option<Scope>,
    client_id: Option<String>,
    client_secret: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IntrospectionRequest {
    pub token: String,
    pub token_type_hint: Option<TokenTypeHint>,
}

#[derive(Debug, Deserialize)]
pub struct RevocationRequest {
    pub token: String,
    pub token_type_hint: Option<TokenTypeHint>,
}
