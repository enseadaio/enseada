use std::sync::Arc;

use enseada::couchdb::db::Database;
use enseada::couchdb::repository::Repository;

use crate::entity::Repo;

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
