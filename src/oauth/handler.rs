use async_trait::async_trait;

use crate::oauth::{RequestHandler, Result};
use crate::oauth::code;
use crate::oauth::scope::Scope;
use crate::oauth::request::{AuthorizationRequest, TokenRequest};
use crate::oauth::error::{ErrorKind, Error};
use crate::oauth::response::{AuthorizationResponse, TokenResponse, TokenType};
use crate::oauth::storage::{ClientStorage, TokenStorage, AuthorizationCodeStorage};
use crate::oauth::token::{AccessToken, RefreshToken};

use std::sync::Arc;
use url::Url;
use crate::secure;


use crate::oauth::session::Session;

use std::collections::HashMap;

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

    async fn validate_client(&self, client_id: &String, _client_secret: Option<String>, redirect_uri: &String, scope: &Scope) -> Result<()> {
        let client_id = client_id.as_str();
        let client = self.client_storage.get_client(client_id).await
            .ok_or_else(|| Error::new(ErrorKind::InvalidClient, format!("client id '{}' is invalid", &client_id)))?;

        // TODO(matteo): if client is confidential, verify client_secret

        scope.matches(client.allowed_scopes())?;

        let uri = Url::parse(&redirect_uri).map_err(|err| Error::new(ErrorKind::InvalidRedirectUri, err.to_string()))?;

        if !client.allowed_redirect_uris().contains(&uri) {
            return Err(Error::new(ErrorKind::InvalidRedirectUri, String::from("invalid redirect URI")));
        }

        Ok(())
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
        self.validate_client(&req.client_id, None, &req.redirect_uri, &req.scope).await
    }

    async fn handle(&self, req: &AuthorizationRequest, session: &mut Session) -> Result<AuthorizationResponse> {
        log::info!("Handling new authorization request");
        session.set_client_id(req.client_id.clone())
            .set_scope(req.scope.clone());

        let secret = secure::generate_token(16).unwrap();
        let code = code::AuthorizationCode::new(secret, session.clone(), 300);
        let code_sig = secure::generate_signature(code.to_string().as_str());
        log::debug!("Storing token with signature {}", code_sig);
        let code = self.authorization_code_storage.store_code(code_sig.to_string().as_str(), code).await?;
        log::debug!("Successfully stored token with signature {}", code_sig);

        let res = AuthorizationResponse::new(code, req.state.as_ref().map(String::clone));
        Ok(res)
    }
}

#[async_trait]
impl<CS, ATS, RTS, ACS> RequestHandler<TokenRequest, TokenResponse> for OAuthHandler<CS, ATS, RTS, ACS>
    where
        CS: ClientStorage,
        ATS: TokenStorage<AccessToken>,
        RTS: TokenStorage<RefreshToken>,
        ACS: AuthorizationCodeStorage
{
    async fn validate(&self, req: &TokenRequest) -> Result<()> {
        match req {
            TokenRequest::AuthorizationCode {
                code, redirect_uri, client_id,
            } => {
                // TODO(matteo): if client_id is None, get from Basic Auth
                let client_id = &client_id.clone().unwrap_or(String::from("example"));

                let code_sig = secure::generate_signature(code.as_str());
                log::debug!("Received auth code with sig {}", &code_sig);
                let code = self.authorization_code_storage.get_code(code_sig.to_string().as_str()).await;
                let code = match code {
                    Some(code) => code,
                    None => return Err(Error::new(ErrorKind::InvalidRequest, "invalid authorization code".to_string())),
                };

                if code.expires_in() < &1 {
                    return Err(Error::new(ErrorKind::InvalidRequest, "invalid authorization code".to_string()));
                }

                let session = code.session();

                if client_id != session.client_id() {
                    return Err(Error::new(ErrorKind::InvalidClient, format!("invalid client '{}'", client_id)));
                }

                self.validate_client(&client_id, None, &redirect_uri, session.scope()).await?;
            }
        };

        Ok(())
    }

    async fn handle(&self, req: &TokenRequest, _session: &mut Session) -> Result<TokenResponse> {
        let res = match req {
            TokenRequest::AuthorizationCode {
                code, redirect_uri: _, client_id: _
            } => {
                let code_sig = secure::generate_signature(code.as_str());
                let code = self.authorization_code_storage.get_code(code_sig.to_string().as_str()).await;
                let code = match code {
                    Some(code) => code,
                    None => return Err(Error::new(ErrorKind::InvalidRequest, "invalid authorization code".to_string())),
                };

                let session = code.session();

                let access_token_value = secure::generate_token(32).unwrap();
                let access_token_sig = secure::generate_signature(access_token_value.to_string().as_str());
                let access_token = AccessToken::new(access_token_value, session.clone(), 3600);
                let access_token = self.access_token_storage.store_token(access_token_sig.to_string().as_str(), access_token).await?;

                let refresh_token_value = secure::generate_token(32).unwrap();
                let refresh_token_sig = secure::generate_signature(refresh_token_value.to_string().as_str());
                let refresh_token = RefreshToken::new(refresh_token_value, session.clone(), 3600);
                let refresh_token = self.refresh_token_storage.store_token(refresh_token_sig.to_string().as_str(), refresh_token).await?;

                self.authorization_code_storage.revoke_code(code_sig.to_string().as_str()).await?;

                TokenResponse {
                    access_token: access_token.to_string(),
                    token_type: TokenType::Bearer,
                    expires_in: access_token.expires_in().clone(),
                    refresh_token: Some(refresh_token.to_string()),
                    scope: session.scope().clone(),
                    extra: HashMap::new(),
                }
            }
        };

        Ok(res)
    }
}