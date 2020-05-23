use actix_web::web;
use serde::{Deserialize, Serialize};

use async_trait::async_trait;

use crate::containers::manifest::oci_v1::{ImageIndex, ImageManifest};
use crate::containers::name::Name;
use crate::containers::Result;
use crate::couchdb;
use crate::couchdb::db::Database;
use crate::couchdb::SINGLETON;
use crate::guid::Guid;

#[async_trait]
pub trait ManifestResolve<L, I> {
    async fn resolve_list(&self, name: &Name) -> Result<Option<L>>;
    async fn resolve_image(&self, name: &Name) -> Result<Option<I>>;
}

pub struct ManifestResolver {
    db: Database,
}

impl ManifestResolver {
    pub fn new(db: Database) -> Self {
        ManifestResolver { db }
    }
}

#[async_trait]
impl ManifestResolve<ImageIndex, ImageManifest> for ManifestResolver {
    async fn resolve_list(&self, name: &Name) -> Result<Option<ImageIndex>> {
        let guid = ImageIndex::build_guid(name);
        let list = self.db.get::<CouchWrapper<ImageIndex>>(&guid.to_string()).await?;
        Ok(list.map(CouchWrapper::into_inner))
    }

    async fn resolve_image(&self, name: &Name) -> Result<Option<ImageManifest>> {
        let guid = ImageManifest::build_guid(name);
        let image = self.db.get::<CouchWrapper<ImageManifest>>(&guid.to_string()).await?;
        Ok(image.map(CouchWrapper::into_inner))
    }
}

pub fn add_manifest_resolver(app: &mut web::ServiceConfig) {
    let db = SINGLETON.database(couchdb::db::name::USERS, true);
    let resolver = ManifestResolver::new(db);
    app.data(resolver);
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CouchWrapper<T> {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    doc: T,
}

impl<T> CouchWrapper<T> {
    pub fn into_inner(self) -> T {
        self.doc
    }
}