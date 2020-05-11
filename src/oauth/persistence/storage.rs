use std::convert::TryInto;
use std::sync::Arc;

use http::StatusCode;

use async_trait::async_trait;

use crate::couchdb;
use crate::couchdb::db::Database;
use crate::oauth::{Expirable, Result};
use crate::oauth::client::Client;
use crate::oauth::code::AuthorizationCode;
use crate::oauth::error::{Error, ErrorKind};
use crate::oauth::persistence::client::ClientEntity;
use crate::oauth::persistence::entity::auth_code::AuthorizationCodeEntity;
use crate::oauth::persistence::entity::token::{AccessTokenEntity, RefreshTokenEntity};
use crate::oauth::storage::{AuthorizationCodeStorage, ClientStorage, TokenStorage};
use crate::oauth::token::{AccessToken, RefreshToken, Token};

pub struct CouchStorage {
    db: Arc<Database>
}

impl CouchStorage {
    pub fn new(db: Arc<Database>) -> CouchStorage {
        CouchStorage { db }
    }

    pub fn save_client(&self, client: Client) -> Result<Client> {
        let entity = ClientEntity::from(client.clone());
        log::info!("{:?}", entity);
        Ok(client)
    }
}

#[async_trait]
impl ClientStorage for CouchStorage {
    async fn get_client(&self, id: &str) -> Option<Client> {
        let guid = ClientEntity::build_guid(&String::from(id));
        let client = match self.db.get::<ClientEntity>(guid.to_string().as_str()).await {
            Ok(client) => match client {
                Some(client) => client,
                None => return None,
            },
            Err(err) => {
                log::error!("Error fetching client from database: {}", err);
                return None;
            }
        };

        client.try_into().ok()
    }
}

#[async_trait]
impl TokenStorage<AccessToken> for CouchStorage {
    async fn get_token(&self, sig: &str) -> Option<AccessToken> {
        let guid = AccessTokenEntity::build_guid(sig);
        let token = match self.db.get::<AccessTokenEntity>(&guid.to_string()).await {
            Ok(token) => token,
            Err(err) => {
                log::error!("Error fetching access token from database: {}", err);
                return None;
            }
        };
        token.map(|t| t.to_empty_token())
    }

    async fn store_token(&self, sig: &str, token: AccessToken) -> Result<AccessToken> {
        let entity = AccessTokenEntity::from_token(sig.to_string(), &token);
        self.db.put(&entity.id().to_string(), &entity).await
            .map_err(map_couch_err)?;
        Ok(entity.to_token(token.token()))
    }

    async fn revoke_token(&self, sig: &str) -> Result<()> {
        let guid = AccessTokenEntity::build_guid(sig);
        let token: Option<AccessTokenEntity> = self.db.get(&guid.to_string()).await
            .map_err(map_couch_err)?;
        match token {
            Some(token) => {
                self.db.delete(token.id().to_string().as_str(), token.rev().unwrap().as_str()).await
                    .map_err(map_couch_err)
            }
            None => Err(Error::new(ErrorKind::InvalidRequest, "invalid access token".to_string()))
        }
    }
}

#[async_trait]
impl TokenStorage<RefreshToken> for CouchStorage {
    async fn get_token(&self, sig: &str) -> Option<RefreshToken> {
        let guid = RefreshTokenEntity::build_guid(sig);
        let token = match self.db.get::<RefreshTokenEntity>(&guid.to_string()).await {
            Ok(token) => token,
            Err(err) => {
                log::error!("Error fetching access token from database: {}", err);
                return None;
            }
        };
        token.map(|t| t.to_empty_token())
    }

    async fn store_token(&self, sig: &str, token: RefreshToken) -> Result<RefreshToken> {
        let entity = RefreshTokenEntity::from_token(sig.to_string(), &token);
        self.db.put(&entity.id().to_string(), &entity).await
            .map_err(map_couch_err)?;
        Ok(entity.to_token(token.token()))
    }

    async fn revoke_token(&self, sig: &str) -> Result<()> {
        let guid = RefreshTokenEntity::build_guid(sig);
        let token: Option<RefreshTokenEntity> = self.db.get(&guid.to_string()).await
            .map_err(map_couch_err)?;
        match token {
            Some(token) => {
                self.db.delete(token.id().to_string().as_str(), token.rev().unwrap().as_str()).await
                    .map_err(map_couch_err)
            }
            None => Err(Error::new(ErrorKind::InvalidRequest, "invalid refresh token".to_string()))
        }
    }
}

#[async_trait]
impl AuthorizationCodeStorage for CouchStorage {
    async fn get_code(&self, sig: &str) -> Option<AuthorizationCode> {
        let guid = AuthorizationCodeEntity::build_guid(sig);
        let code = match self.db.get::<AuthorizationCodeEntity>(&guid.to_string()).await {
            Ok(token) => token,
            Err(err) => {
                log::error!("Error fetching access token from database: {}", err);
                return None;
            }
        };
        code.map(|code| code.to_empty_code())
    }

    async fn store_code(&self, sig: &str, code: AuthorizationCode) -> Result<AuthorizationCode> {
        let entity = AuthorizationCodeEntity::new(String::from(sig), code.session().clone(), *code.expiration());
        self.db.put(&entity.id().to_string(), &entity).await
            .map_err(map_couch_err)?;
        Ok(code)
    }

    async fn revoke_code(&self, sig: &str) -> Result<()> {
        let guid = AuthorizationCodeEntity::build_guid(sig);
        let code: Option<AuthorizationCodeEntity> = self.db.get(&guid.to_string()).await
            .map_err(map_couch_err)?;
        match code {
            Some(code) => {
                self.db.delete(code.id().to_string().as_str(), code.rev().unwrap().as_str()).await
                    .map_err(map_couch_err)
            }
            None => Err(Error::new(ErrorKind::InvalidRequest, "invalid authorization code".to_string()))
        }
    }
}

fn map_couch_err(err: couchdb::error::Error) -> Error {
    Error::new(ErrorKind::ServerError, err.to_string())
}