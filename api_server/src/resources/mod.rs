use std::marker::PhantomData;

use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use api::tonic;
use couchdb::db::Database;
use couchdb::responses::Revs;
pub use watcher::Watcher;

use crate::error::Error;
use crate::resources::id::Id;
use crate::ServerResult;

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
}

#[derive(Clone)]
pub struct ResourceManager<T: Clone + DeserializeOwned + Serialize> {
    db: Database,
    kind: String,
    _phantom: PhantomData<T>,
}

impl<T: Clone + DeserializeOwned + Serialize> ResourceManager<T> {
    pub fn new<K: ToString>(db: Database, kind: K) -> Self {
        Self {
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

    pub async fn find(&self, name: &str) -> Result<Option<T>, tonic::Status> {
        let id = format!("{}:{}", &self.kind, name);
        let ResourceWrapper { resource, .. } = self.db.get::<ResourceWrapper<T>>(&id).await.map_err(map_couch_error)?;
        Ok(Some(resource))
    }

    pub async fn get(&self, name: &str) -> Result<T, tonic::Status> {
        let id = format!("{}:{}", &self.kind, name);
        let ResourceWrapper { resource, .. } = self.db.get::<ResourceWrapper<T>>(&id).await.map_err(map_couch_error)?;
        Ok(resource)
    }

    pub async fn get_deleted(&self, name: &str) -> Result<T, tonic::Status> {
        let id = format!("{}:{}", &self.kind, name);
        let Revs { revisions, .. } = self.db.get_revs(&id).await.map_err(map_couch_error)?;
        let prefix = revisions.start - 1;
        let last_rev = revisions.ids.get(1).unwrap();
        let ResourceWrapper { resource, .. } = self.db.get_at::<ResourceWrapper<T>>(&id, &format!("{}-{}", prefix, last_rev)).await.map_err(map_couch_error)?;
        Ok(resource)
    }

    pub async fn put(&self, name: &str, resource: T) -> Result<T, tonic::Status> {
        let wrapper = ResourceWrapper::new(&self.kind, name, resource);
        self.db.put(&wrapper.id, &wrapper).await.map_err(map_couch_error)?;
        Ok(wrapper.into_inner())
    }

    pub async fn delete(&self, name: &str) -> Result<(), tonic::Status> {
        let kind = &self.kind;
        let id = format!("{}:{}", kind, name);
        let wrapper = self.db.old_get::<ResourceWrapper<T>>(&id).await
            .map_err(map_couch_error)?
            .ok_or_else(|| resource_not_found::<T>(kind, name))?;
        self.db.delete(&id, &wrapper.rev.unwrap()).await.map_err(map_couch_error)
    }

    pub(super) fn db(&self) -> Database {
        self.db.clone()
    }
}

fn resource_not_found<T: Clone + DeserializeOwned + Serialize>(kind: &str, name: &str) -> tonic::Status {
    tonic::Status::not_found(format!("{} \"{}\" not found", kind, name))
}

fn map_couch_error(err: couchdb::error::Error) -> tonic::Status {
    use tonic::Status;

    match err.status() {
        StatusCode::NOT_FOUND => Status::not_found(err.to_string()),
        _ => Status::internal(err.to_string()),
    }
}
