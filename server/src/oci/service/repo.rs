use couchdb::db::Database;

use crate::couchdb::repository::Repository;
use crate::oci::entity::Repo;

pub struct RepoService {
    db: Database,
}

impl RepoService {
    pub fn new(db: Database) -> Self {
        RepoService { db }
    }
}

impl Repository<Repo> for RepoService {
    fn db(&self) -> &Database {
        &self.db
    }
}
