use std::marker::PhantomData;

use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use slog::Logger;

use couchdb::db::Database;
use couchdb::responses::Revs;
pub use watcher::Watcher;

use crate::error::Error;
use crate::resources::id::Id;
use crate::ServerResult;
use api::error::Code;

mod id;
mod watcher;

#[derive(Clone, Deserialize, Serialize)]
#[serde(bound = "T: DeserializeOwned + Serialize")]
pub struct ResourceWrapper<T: DeserializeOwned + Serialize> {
    #[serde(rename = "_id")]
    id: Id,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    resource: T,
}

impl<T: DeserializeOwned + Serialize> ResourceWrapper<T> {
    pub fn new(kind: &str, name: &str, resource: T) -> Self {
        Self {
            id: Id::new(kind, name),
            rev: None,
            resource,
        }
    }

    pub fn into_inner(self) -> T {
        self.resource
    }
    pub fn with_resource(mut self, resource: T) -> Self {
        self.resource = resource;
        self
    }
}

#[derive(Clone)]
pub struct ResourceManager<T: Clone + DeserializeOwned + Serialize> {
    logger: Logger,
    db: Database,
    kind: String,
    _phantom: PhantomData<T>,
}

impl<T: Clone + DeserializeOwned + Serialize> ResourceManager<T> {
    pub fn new<K: ToString>(logger: Logger, db: Database, kind: K) -> Self {
        Self {
            logger,
            db,
            kind: kind.to_string(),
            _phantom: PhantomData::default(),
        }
    }

    pub async fn init(&self) -> ServerResult {
        if let Err(err) = self.db.create_self().await {
            if err.status() != StatusCode::PRECONDITION_FAILED {
                return Err(Error::InitError(format!("failed to initialize database {}: {}", self.db.name(), err)));
            }
        }

        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<T>, Error> {
        let list = self.db.list_partitioned::<ResourceWrapper<T>>(&self.kind, 10, 0).await?;
        Ok(list.rows.into_iter()
            .map(|res| res.doc.unwrap())
            .map(ResourceWrapper::into_inner)
            .collect())
    }

    pub async fn find(&self, name: &str) -> Result<Option<T>, Error> {
        Ok(self.inner_find(name).await?.map(|ResourceWrapper { resource, .. }| resource))
    }

    async fn inner_find(&self, name: &str) -> Result<Option<ResourceWrapper<T>>, Error> {
        let id = format!("{}:{}", &self.kind, name);
        self.db.find_one::<ResourceWrapper<T>>(&id).await.map_err(Error::from)
    }

    pub async fn get(&self, name: &str) -> Result<T, Error> {
        let id = format!("{}:{}", &self.kind, name);
        let ResourceWrapper { resource, .. } = self.db.get::<ResourceWrapper<T>>(&id).await?;
        Ok(resource)
    }

    pub async fn put(&self, name: &str, resource: T) -> Result<T, Error> {
        slog::debug!(self.logger, "Updating resource {}", name);
        let wrapper = self.inner_find(name).await?.map_or_else(
            || ResourceWrapper::new(&self.kind, name, resource.clone()),
            |wrapper| wrapper.with_resource(resource.clone()),
        );
        match &wrapper.rev {
            None => slog::debug!(self.logger, "Creating new resource"),
            Some(rev) => slog::debug!(self.logger, "Updating resource"; "rev" => rev),
        }

        self.db.put(&wrapper.id, &wrapper).await?;
        self.get(&name).await
    }

    pub async fn delete(&self, name: &str) -> Result<(), Error> {
        let kind = &self.kind;
        let id = format!("{}:{}", kind, name);
        let wrapper = self.db.find_one::<ResourceWrapper<T>>(&id).await
            ?
            .ok_or_else(|| resource_not_found::<T>(kind, name))?;
        self.db.delete(&id, &wrapper.rev.unwrap()).await.map_err(Error::from)
    }

    pub(super) fn db(&self) -> Database {
        self.db.clone()
    }
}

fn resource_not_found<T: Clone + DeserializeOwned + Serialize>(kind: &str, name: &str) -> Error {
    Error::ApiError {
        code: Code::NotFound,
        message: format!("{} \"{}\" not found", kind, name),
    }
}
