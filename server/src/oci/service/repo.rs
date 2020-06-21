use std::sync::Arc;

use couchdb::db::Database;

use crate::couchdb::repository::Repository;
use crate::oci::entity::Repo;

#[derive(Debug)]
pub struct RepoService {
    db: Arc<Database>,
}

impl RepoService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

impl Repository<Repo> for RepoService {
    fn db(&self) -> &Database {
        &self.db
    }
}
