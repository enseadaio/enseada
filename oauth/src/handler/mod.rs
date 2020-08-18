use std::collections::HashMap;
use std::sync::Arc;

use chrono::Duration;
use url::Url;

use async_trait::async_trait;
use enseada::secure;

use crate::client::{Client, ClientKind};
use crate::error::{Error, ErrorKind};
use crate::response::{TokenResponse, TokenType, AuthorizationResponse};
use crate::scope::Scope;
use crate::session::Session;
use crate::storage::{AuthorizationCodeStorage, ClientStorage, TokenStorage};
use crate::token::{AccessToken, RefreshToken, Token};
use crate::{Expirable, Result};
use crate::request::{AuthorizationRequest, TokenRequest, IntrospectionRequest};
use crate::code::AuthorizationCode;

mod auth;
mod introspection;
mod revocation;
mod token;

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
    ACS: AuthorizationCodeStorage,
{
    client_storage: Arc<CS>,
    access_token_storage: Arc<ATS>,
    refresh_token_storage: Arc<RTS>,
    authorization_code_storage: Arc<ACS>,
    secret_key: String,
}

impl<CS, ATS, RTS, ACS> OAuthHandler<CS, ATS, RTS, ACS>
where
    CS: ClientStorage,
    ATS: TokenStorage<AccessToken>,
    RTS: TokenStorage<RefreshToken>,
    ACS: AuthorizationCodeStorage,
{
    pub fn new(
        client_storage: Arc<CS>,
        access_token_storage: Arc<ATS>,
        refresh_token_storage: Arc<RTS>,
        authorization_code_storage: Arc<ACS>,
        secret_key: String,
    ) -> OAuthHandler<CS, ATS, RTS, ACS>
    where
        CS: ClientStorage,
        ATS: TokenStorage<AccessToken>,
        RTS: TokenStorage<RefreshToken>,
        ACS: AuthorizationCodeStorage,
    {
        OAuthHandler {
            client_storage,
            access_token_storage,
            refresh_token_storage,
            authorization_code_storage,
            secret_key,
        }
    }

    pub(crate) fn secret_key(&self) -> &str {
        &self.secret_key
    }

    async fn validate_client(
        &self,
        client_id: &str,
        redirect_uri: Option<&str>,
        scope: &Scope,
    ) -> Result<Client> {
        log::debug!("Validating client '{}'", client_id);
        let client = self
            .client_storage
            .get_client(client_id)
            .await
            .ok_or_else(|| Error::new(ErrorKind::InvalidClient, "invalid client_id".to_string()))?;

        log::debug!("Validating request scopes");
        if !client.allowed_scopes().is_superset(scope) {
            return Err(Error::new(
                ErrorKind::InvalidScope,
                "invalid scopes".to_string(),
            ));
        }

        if let Some(redirect_uri) = redirect_uri {
            log::debug!("Validating redirect_uri");
            let uri = Url::parse(&redirect_uri)
                .map_err(|err| Error::new(ErrorKind::InvalidRedirectUri, err.to_string()))?;

            if !client.allowed_redirect_uris().contains(&uri) {
                return Err(Error::new(
                    ErrorKind::InvalidRedirectUri,
                    String::from("invalid redirect URI"),
                ));
            }
        }

        log::debug!("Client validation successful");
        Ok(client)
    }

    async fn authenticate_client(
        &self,
        client: &Client,
        client_secret: Option<&str>,
    ) -> Result<()> {
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
                            Err(Error::new(
                                ErrorKind::InvalidClient,
                                "invalid client credentials".to_string(),
                            ))
                        }
                    }
                    None => Err(Error::new(
                        ErrorKind::InvalidClient,
                        "invalid client credentials".to_string(),
                    )),
                }
            }
        }
    }

    async fn generate_token_set(&self, session: &Session) -> Result<TokenResponse> {
        let access_token_value = secure::generate_token(32).unwrap();
        let access_token_sig =
            secure::generate_signature(access_token_value.to_string().as_str(), self.secret_key())
                .to_string();
        let access_token =
            AccessToken::new(access_token_value, session.clone(), Duration::minutes(5));
        let access_token = self
            .access_token_storage
            .store_token(access_token_sig.as_str(), access_token)
            .await?;

        let refresh_token_value = secure::generate_token(32).unwrap();
        let refresh_token_sig =
            secure::generate_signature(refresh_token_value.to_string().as_str(), self.secret_key());
        let refresh_token = RefreshToken::new(
            refresh_token_value,
            session.clone(),
            Duration::days(1),
            access_token_sig,
        );
        let refresh_token = self
            .refresh_token_storage
            .store_token(refresh_token_sig.to_string().as_str(), refresh_token)
            .await?;

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
impl<CS, ATS, RTS, ACS> TokenIntrospectionHandler<AccessToken> for OAuthHandler<CS, ATS, RTS, ACS>
where
    CS: ClientStorage,
    ATS: TokenStorage<AccessToken>,
    RTS: TokenStorage<RefreshToken>,
    ACS: AuthorizationCodeStorage,
{
    async fn get_token(&self, token: &str) -> Result<AccessToken> {
        let sig = secure::generate_signature(token, self.secret_key());
        let token = self
            .access_token_storage
            .get_token(sig.to_string().as_str())
            .await
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidRequest,
                    "access token not found".to_string(),
                )
            })?;

        Ok(token)
    }

    async fn revoke_token(&self, token: &str) -> Result<()> {
        let sig = secure::generate_signature(token, self.secret_key());
        self.access_token_storage
            .revoke_token(sig.to_string().as_str())
            .await
    }
}

#[async_trait]
impl<CS, ATS, RTS, ACS> TokenIntrospectionHandler<RefreshToken> for OAuthHandler<CS, ATS, RTS, ACS>
where
    CS: ClientStorage,
    ATS: TokenStorage<AccessToken>,
    RTS: TokenStorage<RefreshToken>,
    ACS: AuthorizationCodeStorage,
{
    async fn get_token(&self, token: &str) -> Result<RefreshToken> {
        let sig = secure::generate_signature(token, self.secret_key());
        let token = self
            .refresh_token_storage
            .get_token(sig.to_string().as_str())
            .await
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidRequest,
                    "refresh token not found".to_string(),
                )
            })?;

        Ok(token)
    }

    async fn revoke_token(&self, token: &str) -> Result<()> {
        let sig = secure::generate_signature(token, self.secret_key());
        self.refresh_token_storage
            .revoke_token(sig.to_string().as_str())
            .await
    }
}
