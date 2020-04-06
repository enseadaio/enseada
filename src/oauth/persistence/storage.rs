use async_trait::async_trait;

use crate::oauth::storage::{ClientStorage, TokenStorage, AuthorizationCodeStorage};
use crate::oauth::Result;
use crate::oauth::token::{AccessToken, RefreshToken};
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

    async fn get_code_entity(&self, sig: &str) -> Option<AuthorizationCodeEntity> {
        let res: FindResponse<AuthorizationCodeEntity> = match self.db.find(serde_json::json!({
            "sig": sig,
        })).await.map_err(map_reqwest_err) {
            Ok(res) => res,
            Err(err) => {
                log::error!("{}", err);
                return None;
            }
        };
        if let Some(w) = res.warning {
            log::warn!("{}", w);
        }

        res.docs.first().clone().map(AuthorizationCodeEntity::clone)
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
    async fn get_token(&self, _sig: &str) -> Option<AccessToken> {
        None
    }

    async fn store_token(&self, _sig: &str, token: AccessToken) -> Result<AccessToken> {
        Ok(token)
    }

    async fn revoke_token(&self, _sig: &str) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl TokenStorage<RefreshToken> for CouchStorage {
    async fn get_token(&self, _sig: &str) -> Option<RefreshToken> {
        None
    }

    async fn store_token(&self, _sig: &str, token: RefreshToken) -> Result<RefreshToken> {
        Ok(token)
    }

    async fn revoke_token(&self, _sig: &str) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl AuthorizationCodeStorage for CouchStorage {
    async fn get_code(&self, sig: &str) -> Option<AuthorizationCode> {
        let code = self.get_code_entity(sig).await;
        code.map(|code| code.to_empty_code())
    }

    async fn store_code(&self, sig: &str, code: AuthorizationCode) -> Result<AuthorizationCode> {
        let entity = AuthorizationCodeEntity::new(String::from(sig), code.session().clone());
        self.db.put::<AuthorizationCodeEntity, serde_json::Value>(&entity.id().to_string(), entity).await
            .map_err(map_reqwest_err)?;
        Ok(code)
    }

    async fn revoke_code(&self, sig: &str) -> Result<()> {
        match self.get_code_entity(sig).await {
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