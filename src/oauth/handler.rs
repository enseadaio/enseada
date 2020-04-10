use async_trait::async_trait;

use crate::oauth::{RequestHandler, Result, Expirable};
use crate::oauth::code;
use crate::oauth::scope::Scope;
use crate::oauth::request::{AuthorizationRequest, TokenRequest};
use crate::oauth::error::{ErrorKind, Error};
use crate::oauth::response::{AuthorizationResponse, TokenResponse, TokenType};
use crate::oauth::storage::{ClientStorage, TokenStorage, AuthorizationCodeStorage};
use crate::oauth::token::{AccessToken, RefreshToken, Token};

use std::sync::Arc;
use url::Url;
use crate::secure;


use crate::oauth::session::Session;

use std::collections::HashMap;
use chrono::Duration;

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

    async fn validate_client(&self, client_id: &String, _client_secret: &Option<String>, redirect_uri: &Option<String>, scope: &Scope) -> Result<()> {
        let client_id = client_id.as_str();
        let client = self.client_storage.get_client(client_id).await
            .ok_or_else(|| Error::new(ErrorKind::InvalidClient, format!("client id '{}' is invalid", &client_id)))?;

        // TODO(matteo): if client is confidential, verify client_secret

        scope.matches(client.allowed_scopes())?;

        if let Some(redirect_uri) = redirect_uri {
            let uri = Url::parse(&redirect_uri).map_err(|err| Error::new(ErrorKind::InvalidRedirectUri, err.to_string()))?;

            if !client.allowed_redirect_uris().contains(&uri) {
                return Err(Error::new(ErrorKind::InvalidRedirectUri, String::from("invalid redirect URI")));
            }
        }

        Ok(())
    }

    async fn generate_token_set(&self, session: &Session) -> Result<TokenResponse> {
        let access_token_value = secure::generate_token(32).unwrap();
        let access_token_sig = secure::generate_signature(access_token_value.to_string().as_str());
        let access_token = AccessToken::new(access_token_value, session.clone(), Duration::minutes(5));
        let access_token = self.access_token_storage.store_token(access_token_sig.to_string().as_str(), access_token).await?;

        let refresh_token_value = secure::generate_token(32).unwrap();
        let refresh_token_sig = secure::generate_signature(refresh_token_value.to_string().as_str());
        let refresh_token = RefreshToken::new(refresh_token_value, session.clone(), Duration::days(1));
        let refresh_token = self.refresh_token_storage.store_token(refresh_token_sig.to_string().as_str(), refresh_token).await?;

        Ok(TokenResponse {
            access_token: access_token.to_string(),
            token_type: TokenType::Bearer,
            expires_in: access_token.expires_in(),
            refresh_token: Some(refresh_token.to_string()),
            scope: session.scope().clone(),
            extra: HashMap::new(),
        })
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
        self.validate_client(&req.client_id, &None, &Some(req.redirect_uri.clone()), &req.scope).await
    }

    async fn handle(&self, req: &AuthorizationRequest, session: &mut Session) -> Result<AuthorizationResponse> {
        log::info!("Handling new authorization request");
        session.set_client_id(req.client_id.clone())
            .set_scope(req.scope.clone());

        let secret = secure::generate_token(16).unwrap();
        let code = code::AuthorizationCode::new(secret, session.clone(), Duration::minutes(5));
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
                code, redirect_uri, client_id, client_secret,
            } => {
                // TODO(matteo): if client_id is None, get from Basic Auth
                let client_id = client_id.clone().unwrap_or("".to_string());

                let code_sig = secure::generate_signature(code.as_str());
                log::debug!("Received auth code with sig {}", &code_sig);
                let code = self.authorization_code_storage.get_code(code_sig.to_string().as_str()).await;
                let code = match code {
                    Some(code) => code,
                    None => return Err(Error::new(ErrorKind::InvalidRequest, "invalid authorization code".to_string())),
                };

                if code.is_expired() {
                    return Err(Error::new(ErrorKind::InvalidRequest, "invalid authorization code".to_string()));
                }

                let session = code.session();

                if session.client_id() != &client_id {
                    return Err(Error::new(ErrorKind::InvalidClient, format!("invalid client '{}'", client_id)));
                }

                self.validate_client(&client_id, client_secret, &Some(redirect_uri.clone()), session.scope()).await?;
                Ok(())
            }
            TokenRequest::RefreshToken { refresh_token, scope, client_id, client_secret } => {
                let client_id = client_id.clone().unwrap_or("".to_string()); // TODO(matteo): read from Basic auth
                let refresh_token_sig = secure::generate_signature(refresh_token);
                let refresh_token_sig = &refresh_token_sig.to_string();
                let refresh_token = match self.refresh_token_storage.get_token(refresh_token_sig).await {
                    Some(token) => token,
                    None => return Err(Error::new(ErrorKind::InvalidRequest, "invalid refresh token".to_string()))
                };

                if refresh_token.is_expired() {
                    return Err(Error::new(ErrorKind::UnsupportedGrantType, "invalid refresh token".to_string()));
                }

                let mut session = refresh_token.session().clone();
                if session.client_id() != &client_id {
                    return Err(Error::new(ErrorKind::InvalidClient, "invalid client_id".to_string()));
                }

                if let Some(other) = scope {
                    let scope = session.scope();
                    if !scope.is_superset(other) {
                        return Err(Error::new(ErrorKind::InvalidScope, "invalid scope".to_string()));
                    }

                    session.set_scope(other.clone());
                }

                self.validate_client(&client_id, client_secret, &None, session.scope()).await?;

                Ok(())
            }
            TokenRequest::Unknown => Err(Error::new(ErrorKind::UnsupportedGrantType, "unsupported grant type".to_string()))
        }
    }

    async fn handle(&self, req: &TokenRequest, _session: &mut Session) -> Result<TokenResponse> {
        match req {
            TokenRequest::AuthorizationCode {
                code, redirect_uri: _, client_id: _, client_secret: _
            } => {
                let code_sig = secure::generate_signature(code.as_str());
                let code = self.authorization_code_storage.get_code(code_sig.to_string().as_str()).await;
                let code = match code {
                    Some(code) => code,
                    None => return Err(Error::new(ErrorKind::InvalidRequest, "invalid authorization code".to_string())),
                };

                let session = code.session();

                let res = self.generate_token_set(session).await?;

                self.authorization_code_storage.revoke_code(code_sig.to_string().as_str()).await?;

                Ok(res)
            }
            TokenRequest::RefreshToken { refresh_token, scope, client_id, client_secret } => {
                let refresh_token_sig = secure::generate_signature(refresh_token);
                let refresh_token_sig = &refresh_token_sig.to_string();
                let refresh_token = match self.refresh_token_storage.get_token(refresh_token_sig).await {
                    Some(token) => token,
                    None => return Err(Error::new(ErrorKind::InvalidRequest, "invalid refresh token".to_string()))
                };

                if refresh_token.is_expired() {
                    return Err(Error::new(ErrorKind::UnsupportedGrantType, "invalid refresh token".to_string()));
                }

                let session = refresh_token.session();

                // We revoke it because we are gonna generate a new one anyway
                self.refresh_token_storage.revoke_token(refresh_token_sig).await?;

                self.generate_token_set(session).await
            }
            TokenRequest::Unknown => Err(Error::new(ErrorKind::UnsupportedGrantType, "unsupported grant type".to_string()))
        }
    }
}