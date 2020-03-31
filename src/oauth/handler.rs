use crate::oauth::{RequestHandler, Result};
use crate::oauth::request::AuthorizationRequest;
use crate::oauth::error::{ErrorKind, Error};
use crate::oauth::response::AuthorizationResponse;
use crate::oauth::storage::{ClientStorage, TokenStorage, AuthorizationCodeStorage};
use crate::oauth::token::{AccessToken, RefreshToken};
use crate::oauth::code::AuthorizationCode;
use crate::oauth::client::Client;
use std::sync::Arc;

pub struct OAuthHandler<'a> {
    client_storage: &'a dyn ClientStorage<dyn Client>,
    access_token_storage: &'a dyn TokenStorage<dyn AccessToken>,
    refresh_token_storage: &'a dyn TokenStorage<dyn RefreshToken>,
    authorization_code_storage: &'a dyn AuthorizationCodeStorage<dyn AuthorizationCode>,
}

impl OAuthHandler {
    pub fn builder<'a>() -> OAuthHandlerBuilder<'a> {
        OAuthHandlerBuilder {
            client_storage: None::<&'a dyn ClientStorage<dyn Client>>,
            access_token_storage: None::<&'a dyn TokenStorage<dyn AccessToken>>,
            refresh_token_storage: None::<&'a dyn TokenStorage<dyn RefreshToken>>,
            authorization_code_storage: None::<&'a dyn AuthorizationCodeStorage<dyn AuthorizationCode>>,
        }
    }
}

pub struct OAuthHandlerBuilder<'a> {
    client_storage: Option<&'a dyn ClientStorage<dyn Client>>,
    access_token_storage: Option<&'a dyn TokenStorage<dyn AccessToken>>,
    refresh_token_storage: Option<&'a dyn TokenStorage<dyn RefreshToken>>,
    authorization_code_storage: Option<&'a dyn AuthorizationCodeStorage<dyn AuthorizationCode>>,
}

impl OAuthHandlerBuilder {
    pub fn client_storage<C: Client, CS: ClientStorage<C>>(&mut self, client_storage: &CS) -> &mut Self {
        self.client_storage = Some(client_storage);
        self
    }

    pub fn access_token_storage<A: AccessToken, T: TokenStorage<A>>(&mut self, access_token_storage: &T) -> &mut Self {
        self.access_token_storage = Some(access_token_storage);
        self
    }

    pub fn refresh_token_storage<R: RefreshToken, T: TokenStorage<R>>(&mut self, refresh_token_storage: &T) -> &mut Self {
        self.refresh_token_storage = Some(refresh_token_storage);
        self
    }

    pub fn authorization_code_storage<A: AuthorizationCode, CS: AuthorizationCodeStorage<A>>(&mut self, authorization_code_storage: &CS) -> &mut Self {
        self.authorization_code_storage = Some(authorization_code_storage);
        self
    }

    pub fn build(&self) -> std::result::Result<OAuthHandler, &str> {
        let client_storage = self.client_storage.ok_or("client_storage is required")?;
        let access_token_storage = self.access_token_storage.ok_or("client_storage is required")?;
        let refresh_token_storage = self.refresh_token_storage.ok_or("client_storage is required")?;
        let authorization_code_storage = self.authorization_code_storage.ok_or("client_storage is required")?;
        Ok(OAuthHandler {
            client_storage,
            access_token_storage,
            refresh_token_storage,
            authorization_code_storage,
        })
    }
}

impl RequestHandler<AuthorizationRequest, AuthorizationResponse> for OAuthHandler {
    fn validate(&self, req: &AuthorizationRequest) -> Result<()> {
        let client_id = req.client_id.as_str();
        let client = self.client_storage.get_client(client_id)
            .ok_or_else(|| Error::new(ErrorKind::InvalidClient, format!("client id '{}' is invalid", &client_id)))?;

        let id = client.client_id();
        Ok(())
    }

    fn handle(&self, req: &AuthorizationRequest) -> Result<AuthorizationResponse> {
        AuthorizationResponse::from_req(req)
    }
}