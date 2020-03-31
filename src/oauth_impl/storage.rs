use crate::oauth::storage::{ClientStorage, TokenStorage, AuthorizationCodeStorage};
use crate::oauth_impl::client::OAuthClient;
use crate::oauth_impl::token::{AccessToken, RefreshToken};
use crate::oauth::error::Error;
use crate::oauth::Result;
use crate::oauth_impl::code::AuthorizationCode;

pub struct CouchStorage {
}

impl CouchStorage {
    pub fn new() -> CouchStorage {
        CouchStorage {}
    }
}

impl ClientStorage<OAuthClient> for CouchStorage {
    fn get_client(&self, id: &str) -> Option<OAuthClient> {
        Some(OAuthClient {})
    }
}

impl TokenStorage<AccessToken> for CouchStorage {
    fn get_token(&self, sig: &str) -> Option<AccessToken> {
        Some(AccessToken)
    }

    fn store_token(&self, sig: &str, token: AccessToken) -> Result<AccessToken> {
        Ok(token)
    }

    fn revoke_token(&self, sig: &str) -> Result<()> {
        Ok(())
    }
}

impl TokenStorage<RefreshToken> for CouchStorage {
    fn get_token(&self, sig: &str) -> Option<RefreshToken> {
        Some(RefreshToken)
    }

    fn store_token(&self, sig: &str, token: RefreshToken) -> Result<RefreshToken> {
        Ok(token)
    }

    fn revoke_token(&self, sig: &str) -> Result<()> {
        Ok(())
    }
}

impl AuthorizationCodeStorage<AuthorizationCode> for CouchStorage {
    fn get_code(&self, sig: &str) -> Option<AuthorizationCode> {
        Some(AuthorizationCode)
    }

    fn store_code(&self, sig: &str, code: AuthorizationCode) -> Result<AuthorizationCode> {
        Ok(code)
    }

    fn revoke_code(&self, sig: &str) -> Result<()> {
        Ok(())
    }
}