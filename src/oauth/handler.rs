use std::collections::HashMap;
use std::sync::Arc;

use chrono::Duration;
use url::Url;

use async_trait::async_trait;

use crate::oauth::{Expirable, Result};
use crate::oauth::client::{Client, ClientKind};
use crate::oauth::code;
use crate::oauth::error::{Error, ErrorKind};
use crate::oauth::request::{AuthorizationRequest, TokenRequest};
use crate::oauth::response::{AuthorizationResponse, TokenResponse, TokenType};
use crate::oauth::scope::Scope;
use crate::oauth::session::Session;
use crate::oauth::storage::{AuthorizationCodeStorage, ClientStorage, TokenStorage};
use crate::oauth::token::{AccessToken, RefreshToken, Token};
use crate::secure;

/// Represent HTTP basic authentication as (client_id, client_secret)
#[derive(Debug)]
pub struct BasicAuth(String, Option<String>);

impl BasicAuth {
    pub fn new(username: String, password: Option<String>) -> Self {
        BasicAuth(username, password)
    }
}

#[async_trait]
pub trait RequestHandler<T, R> {
    async fn validate(&self, req: &T, client_auth: Option<&BasicAuth>) -> Result<Client>;
    async fn handle(&self, req: &T, session: &mut Session) -> Result<R>;
}

#[async_trait]
pub trait TokenIntrospectionHandler<T: Token> {
    async fn get_token(&self, token: &str) -> Result<T>;
    async fn revoke_token(&self, token: &str) -> Result<()>;
}

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

    async fn validate_client(&self, client_id: &str, redirect_uri: Option<&String>, scope: &Scope) -> Result<Client> {
        log::debug!("Validating client '{}'", client_id);
        let client = self.client_storage.get_client(client_id).await
            .ok_or_else(|| Error::new(ErrorKind::InvalidClient, "invalid client_id".to_string()))?;

        log::debug!("Validating request scopes");
        if !client.allowed_scopes().is_superset(scope) {
            return Err(Error::new(ErrorKind::InvalidScope, "invalid scopes".to_string()))
        }

        if let Some(redirect_uri) = redirect_uri {
            log::debug!("Validating redirect_uri");
            let uri = Url::parse(&redirect_uri).map_err(|err| Error::new(ErrorKind::InvalidRedirectUri, err.to_string()))?;

            if !client.allowed_redirect_uris().contains(&uri) {
                return Err(Error::new(ErrorKind::InvalidRedirectUri, String::from("invalid redirect URI")));
            }
        }

        log::debug!("Client validation successful");
        Ok(client)
    }

    async fn authenticate_client(&self, client: &Client, client_secret: Option<&String>) -> Result<()> {
        log::debug!("Checking client authentication");
        match client.kind() {
            ClientKind::Public => {
                log::debug!("Client is of kind 'public', no authentication needed");
                Ok(())
            }
            ClientKind::Confidential { secret } => {
                log::debug!("Client is of kind 'confidential', validating secret");
                match client_secret {
                    Some(client_secret) => {
                        if secure::verify_password(secret, client_secret)? {
                            log::debug!("Client authentication successful");
                            Ok(())
                        } else {
                            log::debug!("Client authentication failed");
                            Err(Error::new(ErrorKind::InvalidClient, "invalid client credentials".to_string()))
                        }
                    }
                    None => Err(Error::new(ErrorKind::InvalidClient, "invalid client credentials".to_string())),
                }
            }
        }
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
    async fn validate(&self, req: &AuthorizationRequest, _client_auth: Option<&BasicAuth>) -> Result<Client> {
        self.validate_client(&req.client_id, Some(&req.redirect_uri), &req.scope).await
    }

    async fn handle(&self, req: &AuthorizationRequest, session: &mut Session) -> Result<AuthorizationResponse> {
        log::info!("Handling new authorization request");
        session.set_scope(req.scope.clone());

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
    async fn validate(&self, req: &TokenRequest, client_auth: Option<&BasicAuth>) -> Result<Client> {
        let auth_client_id = client_auth.map(|BasicAuth(client_id, _client_secret)| client_id);
        let auth_client_secret = client_auth.and_then(|BasicAuth(_client_id, client_secret)| client_secret.as_ref());
        match req {
            TokenRequest::AuthorizationCode {
                code, redirect_uri, client_id, client_secret,
            } => {
                log::debug!("Validating AuthorizationCode token request");
                let client_id = client_id.as_ref().or(auth_client_id);
                let client_id = match client_id {
                    Some(client_id) => client_id,
                    None => return Err(Error::new(ErrorKind::InvalidClient, "invalid client_id".to_string())),
                };

                let code_sig = secure::generate_signature(code.as_str());
                log::debug!("Received auth code with sig {}", &code_sig);
                let code = self.authorization_code_storage.get_code(code_sig.to_string().as_str()).await;
                let code = match code {
                    Some(code) => code,
                    None => return Err(Error::new(ErrorKind::InvalidRequest, "invalid authorization code".to_string())),
                };

                if code.is_expired() {
                    log::warn!("Authorization code is expired");
                    return Err(Error::new(ErrorKind::InvalidRequest, "invalid authorization code".to_string()));
                }

                let session = code.session();

                if session.client_id() != client_id {
                    return Err(Error::new(ErrorKind::InvalidClient, format!("invalid client '{}'", client_id)));
                }

                let client = self.validate_client(&client_id, Some(redirect_uri), session.scope()).await?;
                self.authenticate_client(&client, client_secret.as_ref().or(auth_client_secret)).await?;
                Ok(client)
            }
            TokenRequest::RefreshToken { refresh_token, scope, client_id, client_secret } => {
                let client_id = client_id.as_ref().or(auth_client_id);
                let client_id = match client_id {
                    Some(client_id) => client_id,
                    None => return Err(Error::new(ErrorKind::InvalidClient, "invalid client_id".to_string())),
                };
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
                if session.client_id() != client_id {
                    return Err(Error::new(ErrorKind::InvalidClient, "invalid client_id".to_string()));
                }

                if let Some(other) = scope {
                    let scope = session.scope();
                    if !scope.is_superset(other) {
                        return Err(Error::new(ErrorKind::InvalidScope, "invalid scope".to_string()));
                    }

                    session.set_scope(other.clone());
                }

                let client = self.validate_client(&client_id, None, session.scope()).await?;
                self.authenticate_client(&client, client_secret.as_ref().or(auth_client_secret)).await?;
                Ok(client)
            }
            TokenRequest::Unknown => Err(Error::new(ErrorKind::UnsupportedGrantType, "unsupported grant type".to_string()))
        }
    }

    async fn handle(&self, req: &TokenRequest, _session: &mut Session) -> Result<TokenResponse> {
        match req {
            TokenRequest::AuthorizationCode {
                code, ..
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
            TokenRequest::RefreshToken { refresh_token, .. } => {
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

#[async_trait]
impl<CS, ATS, RTS, ACS> TokenIntrospectionHandler<AccessToken> for OAuthHandler<CS, ATS, RTS, ACS>
    where
        CS: ClientStorage,
        ATS: TokenStorage<AccessToken>,
        RTS: TokenStorage<RefreshToken>,
        ACS: AuthorizationCodeStorage
{
    async fn get_token(&self, token: &str) -> Result<AccessToken> {
        let sig = secure::generate_signature(token);
        let token = self.access_token_storage.get_token(sig.to_string().as_str()).await
            .ok_or_else(|| Error::new(ErrorKind::InvalidRequest, "access token not found".to_string()))?;

        Ok(token)
    }

    async fn revoke_token(&self, token: &str) -> Result<()> {
        let sig = secure::generate_signature(token);
        self.access_token_storage.revoke_token(sig.to_string().as_str()).await
    }
}

#[async_trait]
impl<CS, ATS, RTS, ACS> TokenIntrospectionHandler<RefreshToken> for OAuthHandler<CS, ATS, RTS, ACS>
    where
        CS: ClientStorage,
        ATS: TokenStorage<AccessToken>,
        RTS: TokenStorage<RefreshToken>,
        ACS: AuthorizationCodeStorage
{
    async fn get_token(&self, token: &str) -> Result<RefreshToken> {
        let sig = secure::generate_signature(token);
        let token = self.refresh_token_storage.get_token(sig.to_string().as_str()).await
            .ok_or_else(|| Error::new(ErrorKind::InvalidRequest, "refresh token not found".to_string()))?;

        Ok(token)
    }

    async fn revoke_token(&self, token: &str) -> Result<()> {
        let sig = secure::generate_signature(token);
        self.refresh_token_storage.revoke_token(sig.to_string().as_str()).await
    }
}