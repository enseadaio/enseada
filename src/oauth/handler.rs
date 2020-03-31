use crate::oauth::{RequestHandler, Result};
use crate::oauth::request::AuthorizationRequest;
use crate::oauth::error::{ErrorKind, Error};
use crate::oauth::response::AuthorizationResponse;

#[derive(Default)]
pub struct OAuthHandler {

}

impl OAuthHandler {
    pub fn new() -> OAuthHandler {
        Default::default()
    }
}

impl RequestHandler<AuthorizationRequest, AuthorizationResponse> for OAuthHandler {
    fn validate(&self, req: &AuthorizationRequest) -> Result<()> {
        Ok(())
    }

    fn handle(&self, req: &AuthorizationRequest) -> Result<AuthorizationResponse> {
        AuthorizationResponse::from_req(req)
    }
}