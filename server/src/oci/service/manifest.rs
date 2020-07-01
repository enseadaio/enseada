use std::convert::TryFrom;
use std::sync::Arc;

use couchdb::db::Database;

use crate::couchdb::repository::{Entity, Repository};
use crate::oci::digest::Digest;
use crate::oci::entity::Manifest;
use crate::oci::error::Error;
use crate::oci::Result;

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
}

impl Repository<Manifest> for ManifestService {
    fn db(&self) -> &Database {
        self.db.as_ref()
    }
}
