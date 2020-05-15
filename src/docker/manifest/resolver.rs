use actix_web::web;
use serde::{Deserialize, Serialize};

use async_trait::async_trait;

use crate::couchdb;
use crate::couchdb::db::Database;
use crate::couchdb::SINGLETON;
use crate::docker::manifest::s2::{ImageManifest, ManifestList};
use crate::docker::{Result, Name};
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
impl ManifestResolve<ManifestList, ImageManifest> for ManifestResolver {
    async fn resolve_list(&self, name: &Name) -> Result<Option<ManifestList>> {
        let guid = ManifestList::build_guid(name);
        let list = self.db.get(&guid.to_string()).await?;
        Ok(list)
    }

    async fn resolve_image(&self, name: &Name) -> Result<Option<ImageManifest>> {
        let guid = ImageManifest::build_guid(name);
        let image = self.db.get(&guid.to_string()).await?;
        Ok(image)
    }
}

pub fn add_manifest_resolver(app: &mut web::ServiceConfig) {
    let db = SINGLETON.database(couchdb::db::name::OCI, true);
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