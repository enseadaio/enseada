use std::sync::{Arc, RwLock};

use async_trait::async_trait;

use enseada::couchdb::db::Database;
use enseada::couchdb::repository::Repository;
use events::EventBus;

use crate::entity::Repo;
use crate::events::{RepoCreated, RepoDeleted, RepoUpdated};

#[derive(Debug)]
pub struct RepoService {
    db: Arc<Database>,
    bus: Arc<RwLock<EventBus>>,
}

impl RepoService {
    pub fn new(db: Arc<Database>, bus: Arc<RwLock<EventBus>>) -> Self {
        Self { db, bus }
    }
}

#[async_trait]
impl Repository<Repo> for RepoService {
    fn db(&self) -> &Database {
        &self.db
    }

    async fn created(&self, repo: &Repo) {
        let event = RepoCreated::from(repo);
        let bus = self.bus.read().expect("created() EventBus unlock");
        bus.broadcast(event);
    }

    async fn updated(&self, repo: &Repo) {
        let event = RepoUpdated::from(repo);
        let bus = self.bus.read().expect("updated() EventBus unlock");
        bus.broadcast(event);
    }

    async fn deleted(&self, repo: &Repo) {
        let event = RepoDeleted::from(repo);
        let bus = self.bus.read().expect("deleted() EventBus unlock");
        bus.broadcast(event);
    }
}
