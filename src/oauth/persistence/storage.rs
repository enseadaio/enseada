use async_trait::async_trait;

use crate::oauth::storage::{ClientStorage, TokenStorage, AuthorizationCodeStorage};
use crate::oauth::Result;
use crate::oauth::token::{AccessToken, RefreshToken, Token};
use crate::oauth::client::Client;
use crate::oauth::code::AuthorizationCode;
use crate::oauth::persistence::client::ClientEntity;
use std::sync::Arc;
use crate::couchdb::db::Database;
use reqwest::StatusCode;
use crate::oauth::persistence::entity::auth_code::AuthorizationCodeEntity;
use crate::oauth::error::{Error, ErrorKind};
use futures::FutureExt;
use crate::secure::SecureSecret;
use crate::couchdb::responses::FindResponse;
use crate::oauth::persistence::entity::token::{AccessTokenEntity, RefreshTokenEntity};
use chrono::{DateTime, Utc, Duration};
use std::ops::Add;

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
            Ok(client) => client,
            Err(err) => {
                if !err.is_status() || err.status().unwrap() != StatusCode::NOT_FOUND {
                    log::error!("Error fetching client from database: {}", err);
                }

                return None;
            }
        };

        Some(client.into())
    }
}

#[async_trait]
impl TokenStorage<AccessToken> for CouchStorage {
    async fn get_token(&self, sig: &str) -> Option<AccessToken> {
        let guid = AccessTokenEntity::build_guid(String::from(sig));
        let token: Option<AccessTokenEntity> = self.db.get(&guid.to_string()).await.ok();
        token.map(|t| t.to_empty_token())
    }

    async fn store_token(&self, sig: &str, token: AccessToken) -> Result<AccessToken> {
        let expiration = Utc::now().add(Duration::seconds(token.expires_in().clone() as i64));
        let entity = AccessTokenEntity::new(String::from(sig), token.session().clone(), expiration);
        self.db.put::<&AccessTokenEntity, serde_json::Value>(&entity.id().to_string(), &entity).await
            .map_err(map_reqwest_err)?;
        Ok(entity.to_token(token.token()))
    }

    async fn revoke_token(&self, sig: &str) -> Result<()> {
        let guid = AccessTokenEntity::build_guid(String::from(sig));
        let token: Option<AccessTokenEntity> = self.db.get(&guid.to_string()).await
            .map_err(map_reqwest_err)?;
        match token {
            Some(token) => {
                self.db.delete(token.id().to_string().as_str(), token.rev().unwrap().as_str()).await
                    .map_err(map_reqwest_err)
            }
            None => Err(Error::new(ErrorKind::InvalidRequest, "invalid access token".to_string()))
        }
    }
}

#[async_trait]
impl TokenStorage<RefreshToken> for CouchStorage {
    async fn get_token(&self, sig: &str) -> Option<RefreshToken> {
        let guid = RefreshTokenEntity::build_guid(String::from(sig));
        let token: Option<RefreshTokenEntity> = self.db.get(&guid.to_string()).await.ok();
        token.map(|t| t.to_empty_token())
    }

    async fn store_token(&self, sig: &str, token: RefreshToken) -> Result<RefreshToken> {
        let expiration = Utc::now().add(Duration::seconds(token.expires_in().clone() as i64));
        let entity = RefreshTokenEntity::new(String::from(sig), token.session().clone(), expiration);
        self.db.put::<&RefreshTokenEntity, serde_json::Value>(&entity.id().to_string(), &entity).await
            .map_err(map_reqwest_err)?;
        Ok(entity.to_token(token.token()))
    }

    async fn revoke_token(&self, sig: &str) -> Result<()> {
        let guid = RefreshTokenEntity::build_guid(String::from(sig));
        let token: Option<RefreshTokenEntity> = self.db.get(&guid.to_string()).await
            .map_err(map_reqwest_err)?;
        match token {
            Some(token) => {
                self.db.delete(token.id().to_string().as_str(), token.rev().unwrap().as_str()).await
                    .map_err(map_reqwest_err)
            }
            None => Err(Error::new(ErrorKind::InvalidRequest, "invalid refresh token".to_string()))
        }
    }
}

#[async_trait]
impl AuthorizationCodeStorage for CouchStorage {
    async fn get_code(&self, sig: &str) -> Option<AuthorizationCode> {
        let guid = AuthorizationCodeEntity::build_guid(String::from(sig));
        let code: Option<AuthorizationCodeEntity> = self.db.get(&guid.to_string()).await.ok();
        code.map(|code| code.to_empty_code())
    }

    async fn store_code(&self, sig: &str, code: AuthorizationCode) -> Result<AuthorizationCode> {
        let entity = AuthorizationCodeEntity::new(String::from(sig), code.session().clone());
        self.db.put::<&AuthorizationCodeEntity, serde_json::Value>(&entity.id().to_string(), &entity).await
            .map_err(map_reqwest_err)?;
        Ok(code)
    }

    async fn revoke_code(&self, sig: &str) -> Result<()> {
        let guid = AuthorizationCodeEntity::build_guid(String::from(sig));
        let code: Option<AuthorizationCodeEntity> = self.db.get(&guid.to_string()).await
            .map_err(map_reqwest_err)?;
        match code {
            Some(code) => {
                self.db.delete(code.id().to_string().as_str(), code.rev().unwrap().as_str()).await
                    .map_err(map_reqwest_err)
            },
            None => Err(Error::new(ErrorKind::InvalidRequest, "invalid authorization code".to_string()))
        }
    }
}

fn map_reqwest_err(err: reqwest::Error) -> Error {
    Error::new(ErrorKind::ServerError, err.to_string())
}