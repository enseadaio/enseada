use std::sync::{Arc, RwLock};

use async_trait::async_trait;

use enseada::couchdb::db::Database;
use enseada::couchdb::repository::{Entity, Repository};
use enseada::couchdb::responses::{RawDocValue, RowsResponse};
use enseada::couchdb::view::View;
use enseada::pagination::Page;
use events::EventBus;

use crate::entity::{Manifest, Repo};
use crate::error::{Error, ErrorCode};
use crate::events::{RepoCreated, RepoDeleted, RepoUpdated};
use crate::Result;

#[derive(Debug)]
pub struct RepoService {
    db: Arc<Database>,
    bus: Arc<RwLock<EventBus>>,
    tags_view: View,
}

impl RepoService {
    pub fn new(db: Arc<Database>, bus: Arc<RwLock<EventBus>>) -> Self {
        let id = Manifest::build_guid("");
        let part = id.partition().unwrap();
        let tags_view = db.partitioned_view("image_views", part, "image_tags");
        Self { db, bus, tags_view }
    }

    pub async fn list_all_repo_tags(&self, repo: &Repo) -> Result<Vec<String>> {
        let res: RowsResponse<Manifest> = self
            .tags_view
            .list_all_for_key(repo.full_name(), false)
            .await?;
        res.rows
            .iter()
            .map(|row| match &row.value {
                RawDocValue::Rev { .. } => Err(Error::new(
                    ErrorCode::Internal,
                    "image tags view returned a rev value instead of a string",
                )),
                RawDocValue::String(tag) => Ok(tag.clone()),
            })
            .collect()
    }

    pub async fn list_repo_tags(
        &self,
        repo: &Repo,
        limit: usize,
        offset: usize,
    ) -> Result<Page<String>> {
        let res: RowsResponse<Manifest> = self
            .tags_view
            .list_for_key(repo.full_name(), limit, offset, false)
            .await?;
        let tags: Result<Vec<String>> = res
            .rows
            .iter()
            .map(|row| match &row.value {
                RawDocValue::Rev { .. } => Err(Error::new(
                    ErrorCode::Internal,
                    "image tags view returned a rev value instead of a string",
                )),
                RawDocValue::String(tag) => Ok(tag.clone()),
            })
            .collect();

        Ok(Page::from_slice(tags?, limit, offset, res.total_rows))
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
