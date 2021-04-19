use api::Client;
use api::tonic::Status;
use api::tonic::transport::Channel;
use api::users::v1alpha1::users_client::UsersClient;
use api::users::v1alpha1::{WatchUsersRequest, User, UserEvent, UserStatus};

use crate::error::Error;
use slog::Logger;
use crossbeam_channel::Receiver;
use std::time::Duration;
use crate::resources::{Watcher, ResourceManager};
use couchdb::db::Database;
use actix::ArbiterHandle;
use futures::StreamExt;
use api::watch::v1alpha1::EventType;

pub async fn start(logger: Logger, db: Database, arbiter: &ArbiterHandle) -> Result<(), Error> {
    let manager = ResourceManager::new(db, "user");
    manager.init().await?;
    let mut w = Watcher::<User, UserEvent>::start(logger.clone(), manager.clone(), arbiter);

    while let Some(res) = w.next().await {
        let event = res?;
        match EventType::from_i32(event.r#type) {
            Some(EventType::Changed) => on_user_changed(logger.clone(), &manager, event.user.unwrap()).await?,
            Some(EventType::Deleted) => on_user_deleted(logger.clone(), &manager, event.user.unwrap()).await?,
            _ => {}
        }
    }

    Ok(())
}

async fn on_user_changed(logger: Logger, manager: &ResourceManager<User>, mut user: User) -> Result<(), Error> {
    if let Some(spec) = &user.spec {
        let dirty = user.status.as_ref().map_or(true, |status| status.enabled != spec.enabled);
        if dirty {
            user.status = Some(UserStatus { enabled: spec.enabled });
            let name = user.metadata.as_ref().map(|metadata| metadata.name.clone()).unwrap();
            let user = manager.put(&name, user).await?;
            slog::info!(logger, "User {:?} was updated!", user);
        }
    }
    Ok(())
}

async fn on_user_deleted(logger: Logger, manager: &ResourceManager<User>, user: User) -> Result<(), Error> {
    slog::info!(logger, "User {:?} was deleted!", user);
    Ok(())
}
