use std::sync::Arc;

use async_trait::async_trait;

use enseada::couchdb::db::Database;
use enseada::couchdb::repository::Repository;
use events::EventHandler;

use crate::entity::Manifest;
use crate::error::Error;
use crate::events::RepoDeleted;
use crate::Result;

#[derive(Debug)]
pub struct ManifestService {
    db: Arc<Database>,
}

impl ManifestService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn find_by_ref(
        &self,
        group: &str,
        name: &str,
        reference: &str,
    ) -> Result<Option<Manifest>> {
        log::debug!("finding manifest by ref '{}'", reference);
        let id = Manifest::build_id(group, name, reference);
        self.find(&id).await.map_err(Error::from)
    }
}

impl Repository<Manifest> for ManifestService {
    fn db(&self) -> &Database {
        &self.db
    }
}

#[async_trait]
impl EventHandler<RepoDeleted> for ManifestService {
    async fn handle(&self, event: &RepoDeleted) {
        let image = format!("{}/{}", &event.group, &event.name);
        if let Err(err) = self
            .delete_all(serde_json::json!({
              "image": image,
            }))
            .await
        {
            log::error!("{}", err);
        }
    }
}
