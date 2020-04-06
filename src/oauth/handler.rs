use async_trait::async_trait;

use crate::oauth::{RequestHandler, Result};
use crate::oauth::code;
use crate::oauth::request::AuthorizationRequest;
use crate::oauth::error::{ErrorKind, Error};
use crate::oauth::response::AuthorizationResponse;
use crate::oauth::storage::{ClientStorage, TokenStorage, AuthorizationCodeStorage};
use crate::oauth::token::{AccessToken, RefreshToken};

use std::sync::Arc;
use url::Url;
use crate::secure;
use crate::secure::SecureSecret;
use crate::oauth::request::TokenRequest::AuthorizationCode;

pub struct OAuthHandler<CS, ATS, RTS, ACS>
    where
        CS: ClientStorage,
        ATS: TokenStorage<AccessToken>,
        RTS: TokenStorage<RefreshToken>,
        ACS: AuthorizationCodeStorage
{
    client_storage: Arc<CS>,
    access_token_storage: Arc<ATS>,
    refresh_token_storage: Arc<RTS>,
    authorization_code_storage: Arc<ACS>,
}

impl<CS, ATS, RTS, ACS> OAuthHandler<CS, ATS, RTS, ACS>
    where
        CS: ClientStorage,
        ATS: TokenStorage<AccessToken>,
        RTS: TokenStorage<RefreshToken>,
        ACS: AuthorizationCodeStorage
{
    pub fn new(
        client_storage: Arc<CS>,
        access_token_storage: Arc<ATS>,
        refresh_token_storage: Arc<RTS>,
        authorization_code_storage: Arc<ACS>,
    ) -> OAuthHandler<CS, ATS, RTS, ACS>
        where
            CS: ClientStorage,
            ATS: TokenStorage<AccessToken>,
            RTS: TokenStorage<RefreshToken>,
            ACS: AuthorizationCodeStorage
    {
        OAuthHandler {
            client_storage,
            access_token_storage,
            refresh_token_storage,
            authorization_code_storage,
        }
    }
}

#[async_trait]
impl<CS, ATS, RTS, ACS> RequestHandler<AuthorizationRequest, AuthorizationResponse> for OAuthHandler<CS, ATS, RTS, ACS>
    where
        CS: ClientStorage,
        ATS: TokenStorage<AccessToken>,
        RTS: TokenStorage<RefreshToken>,
        ACS: AuthorizationCodeStorage
{
    async fn validate(&self, req: &AuthorizationRequest) -> Result<()> {
        let client_id = req.client_id.as_str();
        let client = self.client_storage.get_client(client_id).await
            .ok_or_else(|| Error::new(ErrorKind::InvalidClient, format!("client id '{}' is invalid", &client_id)))?;

        let scope = &req.scope;
        scope.matches(client.allowed_scopes())?;

        let uri = Url::parse(&req.redirect_uri).map_err(|err| Error::new(ErrorKind::InvalidRedirectUri, err.to_string()))?;

        if !client.allowed_redirect_uris().contains(&uri) {
            return Err(Error::new(ErrorKind::InvalidRedirectUri, String::from("invalid redirect URI")));
        }

        Ok(())
    }

    async fn handle(&self, req: &AuthorizationRequest) -> Result<AuthorizationResponse> {
        let secret = secure::generate_token(16).unwrap();
        let code = code::AuthorizationCode::from(secret);
        let code_sig = secure::generate_signature(code.to_string().as_str());
        let code = self.authorization_code_storage.store_code(code_sig.to_string().as_ref(), code).await?;
        let res = AuthorizationResponse::new(code, req.state.as_ref().map(String::clone));
        Ok(res)
    }
}