use std::sync::{Arc, RwLock};

use async_trait::async_trait;

use enseada::couchdb::db::Database;
use enseada::couchdb::repository::Repository;
use enseada::error::Error;
use enseada::storage::blob::Blob;
use enseada::storage::Provider;
use events::EventBus;
use maven_version::Version;

use crate::entity::Repo;
use crate::file::File;
use crate::storage;
use crate::Result;

#[derive(Debug)]
pub struct RepoService {
    db: Database,
    bus: Arc<RwLock<EventBus>>,
    store: Arc<Provider>,
}

impl RepoService {
    pub fn new(db: Database, bus: Arc<RwLock<EventBus>>, store: Arc<Provider>) -> Self {
        Self { db, bus, store }
    }

    pub async fn find_by_location(&self, location: &str) -> Result<Option<Repo>> {
        let location = location.trim_start_matches('/');
        self.find_one(serde_json::json!({
            "decoded_location": location,
        }))
        .await
        .map_err(Error::from)
    }

    pub async fn is_file_present(
        &self,
        repo: &Repo,
        version: &Version,
        filename: &str,
    ) -> Result<bool> {
        let key = storage::versioned_file_key(repo.location(), version, filename);
        self.store.is_blob_present(&key).await.map_err(Error::from)
    }

    pub async fn get_file<'a, 'f>(
        &self,
        repo: &Repo,
        version: Option<&'f Version>,
        filename: &'f str,
    ) -> Result<File<'f>> {
        let key = match version {
            Some(version) => storage::versioned_file_key(repo.location(), version, filename),
            None => storage::file_key(repo.location(), filename),
        };

        match self.store.get_blob(&key).await? {
            Some(blob) => Ok(File::new(
                version,
                filename,
                blob.size(),
                blob.into_byte_stream(),
            )),
            None => Err(Error::not_found("Maven file", filename)),
        }
    }

    pub async fn store_file<'a, 'f>(&'f self, repo: &Repo, file: File<'f>) -> Result<()> {
        let filename = file.filename();
        let key = match file.version() {
            Some(version) => storage::versioned_file_key(repo.location(), version, filename),
            None => storage::file_key(repo.location(), filename),
        };
        let blob = Blob::new(key, file.size(), file.into_byte_stream());
        self.store.store_blob(blob).await?;
        Ok(())
    }
}

#[async_trait]
impl Repository<Repo> for RepoService {
    fn db(&self) -> &Database {
        &self.db
    }

    async fn deleted(&self, repo: &Repo) {}
}
