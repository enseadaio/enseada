use std::convert::TryFrom;
use std::sync::Arc;

use async_trait::async_trait;
use futures::future::BoxFuture;
use futures::FutureExt;

use enseada::couchdb::db::Database;
use enseada::couchdb::repository::{Entity, Repository};
use events::EventHandler;

use crate::digest::Digest;
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

    pub async fn find_by_ref(&self, reference: &str) -> Result<Option<Manifest>> {
        log::debug!("finding manifest by ref '{}'", reference);
        if let Ok(digest) = Digest::try_from(reference.to_string()) {
            let id = Manifest::build_guid(reference);
            let partition = id.partition().unwrap();
            log::debug!(
                "reference is a digest, looking it up in partition '{}'",
                partition
            );
            let res = self
                .db
                .find_partitioned(
                    partition,
                    serde_json::json!({
                        "manifest.config.digest": digest.to_string()
                    }),
                    1,
                    0,
                )
                .await?;
            if let Some(warn) = &res.warning {
                log::warn!("{}", warn);
            }

            return Ok(res.docs.first().cloned());
        }

        log::debug!("reference is not a digest, looking it as tag");
        self.find(reference).await.map_err(Error::from)
    }

    fn recursively_delete_for_repo<'a>(&'a self, image: &'a str) -> BoxFuture<'a, Result<()>> {
        async move {
            let page = self
                .find_all(
                    100,
                    0,
                    serde_json::json!({
                      "image": image,
                    }),
                )
                .await?;

            for manifest in page.iter() {
                self.delete(manifest).await?;
            }

            if page.is_last() {
                Ok(())
            } else {
                self.recursively_delete_for_repo(image).await
            }
        }
        .boxed()
    }
}

impl Repository<Manifest> for ManifestService {
    fn db(&self) -> &Database {
        self.db.as_ref()
    }
}

#[async_trait]
impl EventHandler<RepoDeleted> for ManifestService {
    async fn handle(&self, event: &RepoDeleted) {
        let image = format!("{}/{}", &event.group, &event.name);
        if let Err(err) = self.recursively_delete_for_repo(&image).await {
            log::error!("{}", err);
        }
    }
}
