use std::fmt::Debug;

use async_trait::async_trait;
use futures::future::BoxFuture;
use futures::prelude::*;
use serde::de::DeserializeOwned;
use serde::Serialize;

use couchdb::db::Database;
use couchdb::error::Error;

use crate::guid::Guid;
use crate::pagination::Page;
use std::pin::Pin;
use futures::stream::BoxStream;

pub trait Entity: Debug + Serialize + DeserializeOwned + Send + Sync {
    fn build_guid(id: &str) -> Guid;

    fn id(&self) -> &Guid;

    fn rev(&self) -> Option<&str>;

    fn set_rev(&mut self, rev: String) -> &mut Self;
}

#[async_trait]
pub trait Repository<T: Entity>: Debug {
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
    async fn list(&self, limit: usize, offset: usize) -> Result<Page<T>, Error>
    where
        Self: Sized,
        T: 'async_trait + Entity,
    {
        let id = T::build_guid("");
        let partition = id.partition();
        let db = self.db();
        let (list, count) = match partition {
            Some(partition) => {
                let list = db.list_partitioned::<T>(partition, limit, offset).await?;
                let count = db.count_partitioned(partition).await?;
                (list, count)
            }
            None => {
                let list = db.list::<T>(limit, offset).await?;
                let count = list.total_rows;
                (list, count)
            }
        };
        Ok(Page::from_rows_response(list, limit, offset, count))
    }

    #[tracing::instrument]
    async fn find_all(
        &self,
        limit: usize,
        offset: usize,
        selector: serde_json::Value,
    ) -> Result<Page<T>, Error>
    where
        Self: Sized,
        T: 'async_trait + Entity,
    {
        let id = T::build_guid("");
        let partition = id.partition().unwrap();
        let db = self.db();
        let res = db
            .find_partitioned(partition, selector, limit, offset)
            .await?;

        if let Some(warning) = &res.warning {
            log::warn!("{}", warning);
        }

        let count = db.count_partitioned(partition).await?;

        Ok(Page::from_find_response(res, limit, offset, count))
    }

    fn find_all_stream(
        &self,
        selector: serde_json::Value,
    ) -> BoxStream<Result<T, Error>>
    where
        Self: Sized,
        T: 'static + Entity,
    {
        let id = T::build_guid("");
        let partition = id.partition().unwrap();
        let db = self.db();
        Box::pin(db.find_partitioned_stream(partition, selector))
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
        let updated = entity.rev().is_some();
        let mut entity = entity;
        if let Some(rev) = self.db().get::<T>(&id).await?.as_ref().and_then(T::rev) {
            entity.set_rev(rev.to_string());
        }
        let res = self.db().put(&id, &entity).await?;
        entity.set_rev(res.rev);
        if updated {
            self.updated(&entity).await;
        } else {
            self.created(&entity).await;
        }
        Ok(entity)
    }

    async fn created(&self, entity: &T) {}

    async fn updated(&self, entity: &T) {}

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
        self.db().delete(&id, &rev).await?;
        self.deleted(&entity).await;
        Ok(())
    }

    #[tracing::instrument]
    fn delete_all(&self, selector: serde_json::Value) -> BoxFuture<Result<(), Error>>
    where
        Self: Sized + Sync,
    {
        async move {
            let page = self.find_all(100, 0, selector.clone()).await?;

            for el in page.iter() {
                self.delete(el).await?;
            }

            if page.is_last() {
                Ok(())
            } else {
                self.delete_all(selector).await
            }
        }
        .boxed()
    }

    async fn deleted(&self, entity: &T) {}
}
