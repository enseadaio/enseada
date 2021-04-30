use std::marker::PhantomData;

use http::StatusCode;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use slog::Logger;

use api::Resource;
use couchdb::db::Database;

use crate::error::ControllerError;
use crate::id::Id;
use chrono::Utc;

#[derive(Clone, Deserialize, Serialize)]
#[serde(bound = "T: DeserializeOwned + Serialize")]
pub struct ResourceWrapper<T: DeserializeOwned + Serialize> {
    #[serde(rename = "_id")]
    id: Id,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    version: String,
    resource: T,
}

impl<T: DeserializeOwned + Serialize> ResourceWrapper<T> {
    pub fn new(kind: &str, version: &str, name: &str, resource: T) -> Self {
        Self {
            id: Id::new(kind, name),
            rev: None,
            version: version.to_string(),
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

    pub fn id(&self) -> &str {
        self.id.as_ref()
    }

    pub fn rev(&self) -> Option<&str> {
        self.rev.as_deref()
    }
}

#[derive(Clone)]
pub struct ResourceManager<T: Resource> {
    logger: Logger,
    db: Database,
    _phantom: PhantomData<T>,
}

impl<T: Resource> ResourceManager<T> {
    pub fn new(logger: Logger, db: Database) -> Self {
        Self {
            logger,
            db,
            _phantom: PhantomData::default(),
        }
    }

    pub async fn init(&self) -> Result<(), ControllerError> {
        if let Err(err) = self.db.create_self().await {
            if err.status() != StatusCode::PRECONDITION_FAILED {
                return Err(ControllerError::InitError(format!("failed to initialize database {}: {}", self.db.name(), err)));
            }
        }

        Ok(())
    }

    pub async fn list(&self, bookmark: Option<String>, limit: usize) -> Result<(Vec<T>, String), ControllerError> {
        let kind = T::type_meta().kind_plural;
        let list = self.db.find_partitioned::<ResourceWrapper<T>>(&kind, serde_json::json!({}), bookmark, limit).await?;
        Ok((list.docs.into_iter()
            .map(ResourceWrapper::into_inner)
            .collect(), list.bookmark))
    }

    pub async fn list_all(&self) -> Result<Vec<T>, ControllerError> {
        let kind = T::type_meta().kind_plural;
        let list = self.db.list_all_partitioned::<ResourceWrapper<T>>(&kind).await?;
        Ok(list.rows.into_iter()
            .map(|res| res.doc.unwrap())
            .map(ResourceWrapper::into_inner)
            .collect())
    }

    pub async fn find(&self, name: &str) -> Result<Option<T>, ControllerError> {
        Ok(self.inner_find(name).await?.map(|ResourceWrapper { resource, .. }| resource))
    }

    async fn inner_find(&self, name: &str) -> Result<Option<ResourceWrapper<T>>, ControllerError> {
        let kind = T::type_meta().kind_plural;
        let id = format!("{}:{}", &kind, name);
        self.db.find_one::<ResourceWrapper<T>>(&id).await.map_err(ControllerError::from)
    }

    pub async fn get(&self, name: &str) -> Result<T, ControllerError> {
        let kind = T::type_meta().kind_plural;
        let id = format!("{}:{}", &kind, name);
        let ResourceWrapper { resource, .. } = self.db.get::<ResourceWrapper<T>>(&id).await?;
        Ok(resource)
    }

    pub async fn put(&self, name: &str, resource: T) -> Result<T, ControllerError> {
        let typ = T::type_meta();
        let kind = typ.kind_plural;
        let version = typ.api_version.version;

        let wrapper = self.inner_find(name).await?.map_or_else(
            || ResourceWrapper::new(&kind, &version, name, resource.clone()),
            |wrapper| wrapper.with_resource(resource.clone()),
        );
        match &wrapper.rev {
            None => slog::debug!(self.logger, "Creating new resource"; "kind" => &kind, "name" => name),
            Some(rev) => slog::debug!(self.logger, "Updating resource"; "kind" => &kind, "name" => name, "rev" => rev),
        }

        self.db.put(&wrapper.id, &wrapper).await?;
        self.get(&name).await
    }

    pub async fn delete(&self, name: &str) -> Result<(), ControllerError> {
        let kind = T::type_meta().kind_plural;
        let id = format!("{}:{}", kind, name);
        let mut wrapper = self.db.find_one::<ResourceWrapper<T>>(&id).await
            ?
            .ok_or_else(|| couchdb::error::Error::not_found(format!("{} \"{}\" not found", kind, name)))?;
        let resource = &mut wrapper.resource;
        resource.metadata_mut().deleted_at = Some(Utc::now());

        self.db.put(&id, wrapper).await?;
        Ok(())
    }

    pub(super) fn db(&self) -> Database {
        self.db.clone()
    }
}
