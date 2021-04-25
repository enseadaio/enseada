use std::error::Error;

pub use::futures::*;
pub use actix::{Arbiter, ArbiterHandle};
pub use async_trait::async_trait;
pub use chrono::*;
use slog::Logger;

use api::Resource;
use couchdb::Couch;
pub use error::*;
pub use manager::*;
pub use watcher::*;

pub mod error;
mod id;
mod manager;
mod watcher;

#[async_trait]
pub trait Reconciler<T: Resource, E: Error = ControllerError> {
    async fn reconcile(&mut self, mut resource: T) -> Result<(), ReconciliationError<E>>;
}

pub type ControllerFactory<T, R> = fn(logger: Logger, manager: ResourceManager<T>) -> R;

pub async fn start_controller<T: 'static + Resource + Unpin, E: Error, R: Reconciler<T, E>>(logger: Logger, couch: Couch, arbiter: &ArbiterHandle, controller_factory: ControllerFactory<T, R>) -> Result<(), ControllerError> {
    let typ = T::type_meta();
    let group = &typ.api_version.group;
    let kind = &typ.kind_plural;

    let db = couch.database(group, true);
    let manager = ResourceManager::new(logger.new(slog::o!("manager" => kind.to_string())), db, kind);
    manager.init().await?;
    let logger = logger.new(slog::o!("controller" => kind.to_string(), "api_version" => typ.api_version.to_string()));
    let mut controller = controller_factory(logger.clone(), manager.clone());
    let mut w = Watcher::<T>::start(logger.clone(), manager.clone(), arbiter, None);

    slog::info!(logger, "Starting controller");
    while let Some(res) = w.next().await {
        let event = match res {
            Ok(e) => e,
            Err(err) => {
                slog::error!(logger, "{}", err);
                continue;
            }
        };

        process_event(logger.clone(), &mut controller, event.resource).await;
    }

    Ok(())
}

async fn process_event<T: Resource, E: Error, R: Reconciler<T, E>>(logger: Logger, controller: &mut R, resource: T) {
    while let Err(err) = controller.reconcile(resource.clone()).await {
        slog::error!(logger, "{}", err);
        if let Some(retry_in) = err.retry_in() {
            tokio::time::sleep(retry_in).await;
        } else {
            break
        }
    }
}
