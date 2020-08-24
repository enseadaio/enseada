use async_trait::async_trait;

use enseada::secure;

use crate::client::Client;
use crate::error::{Error, ErrorKind};
use crate::handler::{BasicAuth, OAuthHandler, RequestHandler};
use crate::request::TokenRequest;
use crate::response::TokenResponse;
use crate::session::Session;
use crate::storage::{AuthorizationCodeStorage, ClientStorage, TokenStorage};
use crate::token::{AccessToken, RefreshToken, Token};
use crate::{Expirable, Result};

#[async_trait]
impl<CS, ATS, RTS, ACS> RequestHandler<TokenRequest, TokenResponse>
    for OAuthHandler<CS, ATS, RTS, ACS>
where
    CS: ClientStorage,
    ATS: TokenStorage<AccessToken>,
    RTS: TokenStorage<RefreshToken>,
    ACS: AuthorizationCodeStorage,
{
    async fn validate(
        &self,
        req: &TokenRequest,
        client_auth: Option<&BasicAuth>,
    ) -> Result<Client> {
        let auth_client_id =
            client_auth.map(|BasicAuth(client_id, _client_secret)| client_id.as_str());
        let auth_client_secret =
            client_auth.and_then(|BasicAuth(_client_id, client_secret)| client_secret.as_deref());
        match req {
            TokenRequest::AuthorizationCode {
                code,
                redirect_uri,
                client_id,
                client_secret,
            } => {
                log::debug!("Validating AuthorizationCode token request");
                let client_id = client_id.as_deref().or(auth_client_id);
                let client_id = match client_id {
                    Some(client_id) => client_id,
                    None => {
                        return Err(Error::new(
                            ErrorKind::InvalidClient,
                            "invalid client_id".to_string(),
                        ))
                    }
                };

                let code_sig = secure::generate_signature(code.as_str(), self.secret_key());
                log::debug!("Received auth code with sig {}", &code_sig);
                let code = self
                    .authorization_code_storage
                    .get_code(code_sig.to_string().as_str())
                    .await;
                let code = match code {
                    Some(code) => code,
                    None => {
                        return Err(Error::new(
                            ErrorKind::InvalidRequest,
                            "invalid authorization code".to_string(),
                        ))
                    }
                };

                if code.is_expired() {
                    log::warn!("Authorization code is expired");
                    return Err(Error::new(
                        ErrorKind::InvalidRequest,
                        "invalid authorization code".to_string(),
                    ));
                }

                let session = code.session();

                if session.client_id() != client_id {
                    return Err(Error::new(
                        ErrorKind::InvalidClient,
                        format!("invalid client '{}'", client_id),
                    ));
                }

                let client = self
                    .validate_client(&client_id, Some(redirect_uri), session.scope())
                    .await?;
                self.authenticate_client(&client, client_secret.as_deref().or(auth_client_secret))
                    .await?;
                Ok(client)
            }
            TokenRequest::RefreshToken {
                refresh_token,
                scope,
                client_id,
                client_secret,
            } => {
                let client_id = client_id.as_deref().or(auth_client_id);
                let client_id = match client_id {
                    Some(client_id) => client_id,
                    None => {
                        return Err(Error::new(
                            ErrorKind::InvalidClient,
                            "invalid client_id".to_string(),
                        ))
                    }
                };
                let refresh_token_sig =
                    secure::generate_signature(refresh_token, self.secret_key());
                let refresh_token_sig = &refresh_token_sig.to_string();
                let refresh_token = match self
                    .refresh_token_storage
                    .get_token(refresh_token_sig)
                    .await
                {
                    Some(token) => token,
                    None => {
                        return Err(Error::new(
                            ErrorKind::InvalidRequest,
                            "invalid refresh token".to_string(),
                        ))
                    }
                };

                if refresh_token.is_expired() {
                    return Err(Error::new(
                        ErrorKind::UnsupportedGrantType,
                        "invalid refresh token".to_string(),
                    ));
                }

                let mut session = refresh_token.session().clone();
                if session.client_id() != client_id {
                    return Err(Error::new(
                        ErrorKind::InvalidClient,
                        "invalid client_id".to_string(),
                    ));
                }

                if let Some(other) = scope {
                    let scope = session.scope();
                    if !scope.is_superset(other) {
                        return Err(Error::new(
                            ErrorKind::InvalidScope,
                            "invalid scope".to_string(),
                        ));
                    }

                    session.set_scope(other.clone());
                }

                let client = self
                    .validate_client(&client_id, None, session.scope())
                    .await?;
                self.authenticate_client(&client, client_secret.as_deref().or(auth_client_secret))
                    .await?;
                Ok(client)
            }
            TokenRequest::Unknown => Err(Error::new(
                ErrorKind::UnsupportedGrantType,
                "unsupported grant type".to_string(),
            )),
        }
    }

    async fn handle(&self, req: &TokenRequest, _session: &mut Session) -> Result<TokenResponse> {
        match req {
            TokenRequest::AuthorizationCode { code, .. } => {
                let code_sig = secure::generate_signature(code.as_str(), self.secret_key());
                let code = self
                    .authorization_code_storage
                    .get_code(code_sig.to_string().as_str())
                    .await;
                let code = match code {
                    Some(code) => code,
                    None => {
                        return Err(Error::new(
                            ErrorKind::InvalidRequest,
                            "invalid authorization code".to_string(),
                        ))
                    }
                };

                let session = code.session();

                let res = self.generate_token_set(session).await?;

                self.authorization_code_storage
                    .revoke_code(code_sig.to_string().as_str())
                    .await?;

                Ok(res)
            }
            TokenRequest::RefreshToken { refresh_token, .. } => {
                let refresh_token_sig =
                    secure::generate_signature(refresh_token, self.secret_key());
                let refresh_token_sig = &refresh_token_sig.to_string();
                let refresh_token = match self
                    .refresh_token_storage
                    .get_token(refresh_token_sig)
                    .await
                {
                    Some(token) => token,
                    None => {
                        return Err(Error::new(
                            ErrorKind::InvalidGrant,
                            "invalid refresh token".to_string(),
                        ))
                    }
                };

                if refresh_token.is_expired() {
                    return Err(Error::new(
                        ErrorKind::UnsupportedGrantType,
                        "invalid refresh token".to_string(),
                    ));
                }

                let session = refresh_token.session();

                // We revoke it because we are gonna generate a new one anyway
                self.refresh_token_storage
                    .revoke_token(refresh_token_sig)
                    .await?;
                // We don't care if the revocation fails, since the access token may have been revoked before the refresh token.
                self.access_token_storage
                    .revoke_token(refresh_token.related_access_token_signature())
                    .await
                    .ok();
                self.generate_token_set(session).await
            }
            TokenRequest::Unknown => Err(Error::new(
                ErrorKind::UnsupportedGrantType,
                "unsupported grant type".to_string(),
            )),
        }
    }
}
