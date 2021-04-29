use async_trait::async_trait;
use chrono::Duration;

use enseada::secure;

use crate::client::Client;
use crate::code;
use crate::error::Error;
use crate::handler::{BasicAuth, OAuthHandler, RequestHandler};
use crate::request::AuthorizationRequest;
use crate::response::AuthorizationResponse;
use crate::session::Session;
use crate::storage::{AuthorizationCodeStorage, ClientStorage, TokenStorage};
use crate::token::{AccessToken, RefreshToken};
use crate::Result;

#[async_trait]
impl<CS, ATS, RTS, ACS> RequestHandler<AuthorizationRequest, AuthorizationResponse>
    for OAuthHandler<CS, ATS, RTS, ACS>
where
    CS: ClientStorage,
    ATS: TokenStorage<AccessToken>,
    RTS: TokenStorage<RefreshToken>,
    ACS: AuthorizationCodeStorage,
{
    async fn validate(
        &self,
        req: &AuthorizationRequest,
        _client_auth: Option<&BasicAuth>,
    ) -> Result<Client> {
        self.validate_client(&req.client_id, Some(&req.redirect_uri), &req.scope)
            .await
            .and_then(|client| {
                if let Some(pkce) = &req.pkce {
                    pkce.validate(req.state.as_deref())?;
                }
                Ok(client)
            })
            .map_err(|mut err| {
                err.set_state(req.state.as_deref());
                err
            })
    }

    async fn handle(
        &self,
        req: &AuthorizationRequest,
        session: &mut Session,
    ) -> Result<AuthorizationResponse> {
        log::info!("Handling new authorization request");
        session.set_scope(req.scope.clone());

        let secret = secure::generate_token(16).unwrap();
        let code = code::AuthorizationCode::new(
            secret,
            session.clone(),
            Duration::minutes(5),
            req.pkce.clone(),
        );
        let code_sig = secure::generate_signature(code.to_string().as_str(), self.secret_key());
        log::debug!("Storing token with signature {}", code_sig);
        let code = self
            .authorization_code_storage
            .store_code(code_sig.to_string().as_str(), code)
            .await
            .map_err(Error::from)
            .map_err(|mut err| {
                err.set_state(req.state.as_deref());
                err
            })?;
        log::debug!("Successfully stored token with signature {}", code_sig);

        let res = AuthorizationResponse::new(code, req.state.clone());
        Ok(res)
    }
}
