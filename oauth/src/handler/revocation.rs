use async_trait::async_trait;

use enseada::secure;

use crate::client::Client;
use crate::error::{Error, ErrorKind};
use crate::handler::{BasicAuth, OAuthHandler, RequestHandler};
use crate::request::RevocationRequest;
use crate::response::RevocationResponse;
use crate::session::Session;
use crate::storage::{AuthorizationCodeStorage, ClientStorage, TokenStorage};
use crate::token::{AccessToken, RefreshToken, Token, TokenTypeHint};
use crate::Result;

#[async_trait]
impl<CS, ATS, RTS, ACS> RequestHandler<RevocationRequest, RevocationResponse>
    for OAuthHandler<CS, ATS, RTS, ACS>
where
    CS: ClientStorage,
    ATS: TokenStorage<AccessToken>,
    RTS: TokenStorage<RefreshToken>,
    ACS: AuthorizationCodeStorage,
{
    async fn validate(
        &self,
        _req: &RevocationRequest,
        client_auth: Option<&BasicAuth>,
    ) -> Result<Client> {
        let BasicAuth(client_id, client_secret) = client_auth.ok_or_else(|| {
            Error::new(
                ErrorKind::AccessDenied,
                "revocation requires client credentials".to_string(),
            )
        })?;
        let client = self
            .client_storage
            .get_client(client_id)
            .await
            .ok_or_else(|| Error::new(ErrorKind::InvalidClient, "invalid client_id".to_string()))?;
        self.authenticate_client(&client, client_secret.as_deref())
            .await?;
        Ok(client)
    }

    async fn handle(
        &self,
        req: &RevocationRequest,
        session: &mut Session,
    ) -> Result<RevocationResponse> {
        let requester_client_id = session.client_id();
        let ok = RevocationResponse::ok();
        let sig = secure::generate_signature(&req.token, self.secret_key()).to_string();
        let sig = sig.as_str();
        if let Some(hint) = &req.token_type_hint {
            if let Some(()) = match hint {
                TokenTypeHint::AccessToken => {
                    let access_token = self.access_token_storage.get_token(sig).await;
                    if let Some(access_token) = access_token {
                        let session = access_token.session();
                        if session.client_id() != requester_client_id {
                            return Err(Error::new(
                                ErrorKind::AccessDenied,
                                "access denied".to_string(),
                            ));
                        }
                        self.access_token_storage.revoke_token(sig).await?;
                    }
                    Some(())
                }
                TokenTypeHint::RefreshToken => {
                    let refresh_token = match self.refresh_token_storage.get_token(sig).await {
                        Some(token) => token,
                        None => return Ok(ok),
                    };

                    let session = refresh_token.session();
                    if session.client_id() != requester_client_id {
                        return Err(Error::new(
                            ErrorKind::AccessDenied,
                            "access denied".to_string(),
                        ));
                    }
                    self.refresh_token_storage.revoke_token(sig).await?;
                    if self
                        .access_token_storage
                        .revoke_token(refresh_token.related_access_token_signature())
                        .await
                        .ok()
                        .is_none()
                    {}
                    Some(())
                }
                TokenTypeHint::Unknown => None,
            } {
                return Ok(ok);
            };
        };

        let access_token = self.access_token_storage.get_token(sig).await;
        if let Some(access_token) = access_token {
            let session = access_token.session();
            if session.client_id() != requester_client_id {
                return Err(Error::new(
                    ErrorKind::AccessDenied,
                    "access denied".to_string(),
                ));
            }
            self.access_token_storage.revoke_token(sig).await;
            return Ok(ok);
        }

        if let Some(refresh_token) = self.refresh_token_storage.get_token(sig).await {
            let session = refresh_token.session();
            if session.client_id() != requester_client_id {
                return Err(Error::new(
                    ErrorKind::AccessDenied,
                    "access denied".to_string(),
                ));
            }
            self.refresh_token_storage.revoke_token(sig).await?;
            // We don't care if the revocation fails, since the access token may have been revoked before the refresh token.
            self.access_token_storage
                .revoke_token(refresh_token.related_access_token_signature())
                .await
                .ok();
            return Ok(ok);
        };

        Ok(ok)
    }
}
