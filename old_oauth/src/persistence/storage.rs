use std::convert::TryInto;
use std::ops::Deref;
use std::sync::Arc;

use async_trait::async_trait;

use enseada::couchdb::db::Database;
use enseada::couchdb::repository::{Entity, Repository};
use enseada::guid::Guid;
use enseada::pagination::Page;
use events::EventHandler;
use users::events::UserDeleted;

use crate::client::Client;
use crate::code::AuthorizationCode;
use crate::error::{Error, ErrorKind};
use crate::persistence::client::ClientEntity;
use crate::persistence::entity::auth_code::{AuthorizationCodeEntity, PkceRequestEntity};
use crate::persistence::entity::pat::PersonalAccessToken;
use crate::persistence::entity::token::{AccessTokenEntity, RefreshTokenEntity};
use crate::storage::{AuthorizationCodeStorage, ClientStorage, TokenStorage};
use crate::token::{AccessToken, RefreshToken, Token};
use crate::{Expirable, Result};

#[derive(Debug)]
pub struct CouchStorage {
    db: Database,
}

impl CouchStorage {
    pub fn new(db: Database) -> CouchStorage {
        CouchStorage { db }
    }
}

#[async_trait]
impl ClientStorage for CouchStorage {
    async fn list_clients(&self, limit: usize, offset: usize) -> Result<Page<Client>> {
        let res = self
            .db
            .list_partitioned::<ClientEntity>("client", limit, offset)
            .await?;
        let count = self.db.count_partitioned("client").await?;
        Ok(Page::from_rows_response(res, limit, offset, count)
            .map(|entity| ClientEntity::try_into(entity.clone()).unwrap()))
    }

    async fn get_client(&self, id: &str) -> Option<Client> {
        let guid = ClientEntity::build_guid(id);
        let client = match self.db.old_get::<ClientEntity>(guid.to_string().as_str()).await {
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

    async fn save_client(&self, client: Client) -> Result<Client> {
        let id = ClientEntity::build_guid(client.client_id());
        let mut entity = ClientEntity::from(client.clone());
        if let Some(rev) = self
            .db
            .old_get::<ClientEntity>(&id.to_string())
            .await?
            .as_ref()
            .and_then(ClientEntity::rev)
        {
            entity.set_rev(rev.to_string());
        }
        let res = self.db.put(&entity.id().to_string(), &entity).await?;
        entity.set_rev(res.rev);
        entity.try_into()
    }

    async fn delete_client(&self, client: &Client) -> Result<()> {
        let id = ClientEntity::build_guid(client.client_id());
        let entity = self.db.old_get::<ClientEntity>(&id.to_string()).await?;
        let entity = entity.ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidClient,
                format!("client '{}' not found", id.id()),
            )
        })?;
        self.db
            .delete(&entity.id().to_string(), entity.rev().unwrap())
            .await?;
        Ok(())
    }
}

#[async_trait]
impl TokenStorage<AccessToken> for CouchStorage {
    async fn get_token(&self, sig: &str) -> Option<AccessToken> {
        let guid = AccessTokenEntity::build_guid(sig);
        let token = match self.db.old_get::<AccessTokenEntity>(&guid.to_string()).await {
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
        self.db
            .put(&entity.id().to_string(), &entity)
            .await
            .map_err(map_couch_err)?;
        Ok(entity.to_token(token.token()))
    }

    async fn revoke_token(&self, sig: &str) -> Result<()> {
        let guid = AccessTokenEntity::build_guid(sig);
        let token: Option<AccessTokenEntity> = self
            .db
            .old_get(&guid.to_string())
            .await
            .map_err(map_couch_err)?;
        match token {
            Some(token) => self
                .db
                .delete(token.id().to_string().as_str(), token.rev().unwrap())
                .await
                .map_err(map_couch_err),
            None => Err(Error::new(
                ErrorKind::InvalidRequest,
                "invalid access token".to_string(),
            )),
        }
    }
}

#[async_trait]
impl TokenStorage<RefreshToken> for CouchStorage {
    async fn get_token(&self, sig: &str) -> Option<RefreshToken> {
        let guid = RefreshTokenEntity::build_guid(sig);
        let token = match self.db.old_get::<RefreshTokenEntity>(&guid.to_string()).await {
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
        self.db
            .put(&entity.id().to_string(), &entity)
            .await
            .map_err(map_couch_err)?;
        Ok(entity.to_token(token.token()))
    }

    async fn revoke_token(&self, sig: &str) -> Result<()> {
        let guid = RefreshTokenEntity::build_guid(sig);
        let token: Option<RefreshTokenEntity> = self
            .db
            .old_get(&guid.to_string())
            .await
            .map_err(map_couch_err)?;
        match token {
            Some(token) => self
                .db
                .delete(token.id().to_string().as_str(), token.rev().unwrap())
                .await
                .map_err(map_couch_err),
            None => Err(Error::new(
                ErrorKind::InvalidRequest,
                "invalid refresh token".to_string(),
            )),
        }
    }
}

#[async_trait]
impl AuthorizationCodeStorage for CouchStorage {
    async fn get_code(&self, sig: &str) -> Option<AuthorizationCode> {
        let guid = AuthorizationCodeEntity::build_guid(sig);
        let code = match self
            .db
            .old_get::<AuthorizationCodeEntity>(&guid.to_string())
            .await
        {
            Ok(token) => token,
            Err(err) => {
                log::error!("Error fetching access token from database: {}", err);
                return None;
            }
        };
        code.map(AuthorizationCodeEntity::into_anonymous_code)
    }

    async fn store_code(&self, sig: &str, code: AuthorizationCode) -> Result<AuthorizationCode> {
        let pkce = code.pkce().cloned().map(PkceRequestEntity::from);
        let entity = AuthorizationCodeEntity::new(
            String::from(sig),
            code.session().clone(),
            code.expiration(),
            pkce,
        );
        self.db
            .put(&entity.id().to_string(), &entity)
            .await
            .map_err(map_couch_err)?;
        Ok(code)
    }

    async fn revoke_code(&self, sig: &str) -> Result<()> {
        let guid = AuthorizationCodeEntity::build_guid(sig);
        let code: Option<AuthorizationCodeEntity> = self
            .db
            .old_get(&guid.to_string())
            .await
            .map_err(map_couch_err)?;
        match code {
            Some(code) => self
                .db
                .delete(code.id().to_string().as_str(), code.rev().unwrap())
                .await
                .map_err(map_couch_err),
            None => Err(Error::new(
                ErrorKind::InvalidRequest,
                "invalid authorization code".to_string(),
            )),
        }
    }
}

impl Repository<PersonalAccessToken> for CouchStorage {
    fn db(&self) -> &Database {
        &self.db
    }
}

#[async_trait]
impl EventHandler<UserDeleted> for CouchStorage {
    async fn handle(&self, event: &UserDeleted) {
        let user_id = &event.id;
        if let Err(err) = self
            .delete_all(serde_json::json!({
                "session.user_id": user_id,
            }))
            .await
        {
            log::error!("{}", err);
        }
    }
}

fn map_couch_err(err: enseada::couchdb::error::Error) -> Error {
    Error::new(ErrorKind::ServerError, err.to_string())
}
