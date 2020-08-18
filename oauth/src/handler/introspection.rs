use async_trait::async_trait;

use enseada::secure;

use crate::client::Client;
use crate::error::{Error, ErrorKind};
use crate::handler::{BasicAuth, OAuthHandler, RequestHandler};
use crate::request::IntrospectionRequest;
use crate::response::IntrospectionResponse;
use crate::session::Session;
use crate::storage::{AuthorizationCodeStorage, ClientStorage, TokenStorage};
use crate::token::{AccessToken, RefreshToken, TokenTypeHint};
use crate::Result;

#[async_trait]
impl<CS, ATS, RTS, ACS> RequestHandler<IntrospectionRequest, IntrospectionResponse>
    for OAuthHandler<CS, ATS, RTS, ACS>
where
    CS: ClientStorage,
    ATS: TokenStorage<AccessToken>,
    RTS: TokenStorage<RefreshToken>,
    ACS: AuthorizationCodeStorage,
{
    async fn validate(
        &self,
        _req: &IntrospectionRequest,
        client_auth: Option<&BasicAuth>,
    ) -> Result<Client> {
        let BasicAuth(client_id, client_secret) = client_auth.ok_or_else(|| {
            Error::new(
                ErrorKind::AccessDenied,
                "introspection requires client credentials".to_string(),
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
        req: &IntrospectionRequest,
        _session: &mut Session,
    ) -> Result<IntrospectionResponse> {
        let sig = secure::generate_signature(&req.token, self.secret_key()).to_string();
        let sig = sig.as_str();
        if let Some(hint) = &req.token_type_hint {
            if let Some(res) = match hint {
                TokenTypeHint::AccessToken => self
                    .access_token_storage
                    .get_token(sig)
                    .await
                    .as_ref()
                    .map(IntrospectionResponse::from_token),
                TokenTypeHint::RefreshToken => self
                    .refresh_token_storage
                    .get_token(sig)
                    .await
                    .as_ref()
                    .map(IntrospectionResponse::from_token),
                TokenTypeHint::Unknown => None,
            } {
                return Ok(res);
            };
        };

        let access_token = self
            .access_token_storage
            .get_token(sig)
            .await
            .as_ref()
            .map(IntrospectionResponse::from_token);
        if let Some(res) = access_token {
            return Ok(res);
        }

        let refresh_token = self
            .refresh_token_storage
            .get_token(sig)
            .await
            .as_ref()
            .map(IntrospectionResponse::from_token);
        if let Some(res) = refresh_token {
            return Ok(res);
        }

        Ok(IntrospectionResponse::inactive())
    }
}
