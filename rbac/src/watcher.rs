use std::sync::Arc;

use actix_rt::Arbiter;
use futures::StreamExt;
use tokio::sync::RwLock;

use enseada::couchdb::changes::ChangeEvent;
use enseada::couchdb::db::Database;
use enseada::error::Error;

use crate::enforcer::Enforcer;

pub struct Watcher {
    db: Database,
    arbiter: Arbiter,
    enforcer: Arc<RwLock<Enforcer>>,
}

impl Watcher {
    pub fn new(db: Database, enforcer: Arc<RwLock<Enforcer>>) -> Self {
        Watcher {
            db,
            arbiter: Arbiter::new(),
            enforcer,
        }
    }

    pub fn start(&self) -> Result<(), Error> {
        let db = self.db.clone();
        let enf = self.enforcer.clone();
        let fut = Box::pin(async move {
            loop {
                log::trace!("Getting fresh change stream");
                match db.changes().await {
                    Ok(mut stream) => {
                        while let Some(el) = stream.next().await {
                            match el {
                                ChangeEvent::Next { .. } => {
                                    log::trace!(
                                        "Received change event from database. Reloading module"
                                    );
                                    let mut enf = enf.write().await;
                                    if let Err(err) = enf.load_rules().await {
                                        log::error!("Failed to reload RBAC rules: {:?}", &err);
                                    }
                                }
                                ChangeEvent::End { .. } => {
                                    continue;
                                }
                            }
                        }
                    }
                    Err(err) => {
                        log::error!("{:?}", err);
                    }
                }
            }
        });
        self.arbiter.send(fut);

        Ok(())
    }

    pub fn stop(&self) {
        self.arbiter.stop();
    }
}
