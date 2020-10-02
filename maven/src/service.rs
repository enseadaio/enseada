use std::sync::{Arc, RwLock};

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

    pub async fn file_exists(
        &self,
        repo: &Repo,
        version: &Version,
        filename: &str,
    ) -> Result<bool> {
        let key = storage::file_key(repo.location(), version, filename);
        self.store.is_blob_present(&key).await.map_err(Error::from)
    }

    pub async fn get_file<'a, 'f>(
        &self,
        repo: &Repo,
        version: &'f Version,
        filename: &'f str,
    ) -> Result<File<'f>> {
        let key = storage::file_key(repo.location(), version, filename);
        match self.store.get_blob(&key).await? {
            Some(blob) => Ok(File::new(filename, version, vec![])),
            None => Err(Error::not_found("Maven file", &key)),
        }
    }

    pub async fn store_file<'a, 'f>(&'f self, repo: &Repo, file: File<'f>) -> Result<()> {
        let key = storage::file_key(repo.location(), file.version(), file.filename());
        // let blob = Blob::new(key, file.into_content());
        // self.store.store_blob(blob).await?;
        Ok(())
    }
}

impl Repository<Repo> for RepoService {
    fn db(&self) -> &Database {
        &self.db
    }
}
