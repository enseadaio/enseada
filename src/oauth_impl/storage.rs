use crate::oauth::storage::{ClientStorage, TokenStorage, AuthorizationCodeStorage};

use crate::oauth::Result;
use crate::oauth::token::{AccessToken, RefreshToken};
use crate::oauth::client::Client;
use crate::oauth::code::AuthorizationCode;

pub struct CouchStorage {
}

impl CouchStorage {
    pub fn new() -> CouchStorage {
        CouchStorage {}
    }
}

impl ClientStorage for CouchStorage {
    fn get_client(&self, _id: &str) -> Option<Client> {
        None
    }
}

impl TokenStorage<AccessToken> for CouchStorage {
    fn get_token(&self, _sig: &str) -> Option<AccessToken> {
        None
    }

    fn store_token(&self, _sig: &str, token: AccessToken) -> Result<AccessToken> {
        Ok(token)
    }

    fn revoke_token(&self, _sig: &str) -> Result<()> {
        Ok(())
    }
}

impl TokenStorage<RefreshToken> for CouchStorage {
    fn get_token(&self, _sig: &str) -> Option<RefreshToken> {
        None
    }

    fn store_token(&self, _sig: &str, token: RefreshToken) -> Result<RefreshToken> {
        Ok(token)
    }

    fn revoke_token(&self, _sig: &str) -> Result<()> {
        Ok(())
    }
}

impl AuthorizationCodeStorage for CouchStorage {
    fn get_code(&self, _sig: &str) -> Option<AuthorizationCode> {
        None
    }

    fn store_code(&self, _sig: &str, code: AuthorizationCode) -> Result<AuthorizationCode> {
        Ok(code)
    }

    fn revoke_code(&self, _sig: &str) -> Result<()> {
        Ok(())
    }
}