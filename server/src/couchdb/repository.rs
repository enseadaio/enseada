use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::Serialize;

use async_trait::async_trait;
use couchdb::db::Database;
use couchdb::error::Error;
use enseada::guid::Guid;
use enseada::pagination::{Cursor, Page};

pub trait Entity: Clone + Debug + Serialize + DeserializeOwned + Send + Sync {
    fn build_guid(id: &str) -> Guid;

    fn id(&self) -> &Guid;

    fn rev(&self) -> Option<&str>;

    fn set_rev(&mut self, rev: String) -> &mut Self;
}

#[async_trait]
pub trait Repository<T>: Debug {
    fn db(&self) -> &Database;

    #[tracing::instrument]
    async fn count(&self) -> Result<usize, Error>
    where
        Self: Sized,
        T: 'async_trait + Entity,
    {
        let guid = T::build_guid("");
        let db = self.db();
        match guid.partition() {
            Some(partition) => db.count_partitioned(partition).await,
            None => db.count().await,
        }
        .map_err(Error::from)
    }

    #[tracing::instrument]
    async fn list(&self, limit: usize, cursor: Option<&Cursor>) -> Result<Page<T>, Error>
    where
        Self: Sized,
        T: 'async_trait + Entity,
    {
        let id = T::build_guid("");
        let partition = id.partition();
        let db = self.db();
        let res = match partition {
            Some(partition) => {
                db.list_partitioned::<T>(partition, limit + 1, cursor.map(Cursor::to_string))
                    .await?
            }
            None => {
                db.list::<T>(limit + 1, cursor.map(Cursor::to_string))
                    .await?
            }
        };
        Ok(Page::from_rows_response(res, limit))
    }

    #[tracing::instrument]
    async fn find(&self, id: &str) -> Result<Option<T>, Error>
    where
        Self: Sized,
        T: 'async_trait + Entity,
    {
        let guid = T::build_guid(id).to_string();
        self.db().get(guid.as_str()).await.map_err(Error::from)
    }

    #[tracing::instrument]
    async fn get(&self, id: &str) -> Result<T, Error>
    where
        Self: Sized,
        T: 'async_trait + Entity,
    {
        self.find(id)
            .await?
            .ok_or_else(|| Error::not_found(format!("entity {} not found", id)))
    }

    #[tracing::instrument]
    async fn save(&self, entity: T) -> Result<T, Error>
    where
        Self: Sized,
        T: 'async_trait + Entity,
    {
        let id = entity.id().to_string();
        let mut entity = entity;
        if let Some(rev) = self.db().get::<T>(&id).await?.as_ref().and_then(T::rev) {
            entity.set_rev(rev.to_string());
        }
        let res = self.db().put(&id, &entity).await?;
        entity.set_rev(res.rev);
        Ok(entity)
    }

    #[tracing::instrument]
    async fn delete(&self, entity: &T) -> Result<(), Error>
    where
        Self: Sized,
        T: 'async_trait + Entity,
    {
        let id = entity.id().to_string();
        let rev = match entity.rev() {
            Some(rev) => rev,
            None => panic!("entity {} is missing rev", id),
        };
        self.db().delete(&id, &rev).await.map_err(Error::from)
    }
}
