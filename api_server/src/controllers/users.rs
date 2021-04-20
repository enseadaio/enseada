use actix::ArbiterHandle;
use chrono::Utc;
use futures::StreamExt;
use slog::Logger;

use api::users::v1alpha1::{User, UserStatus};
use couchdb::Couch;

use crate::error::Error;
use crate::resources::{ResourceManager, Watcher};

pub async fn start(logger: Logger, couch: Couch, arbiter: &ArbiterHandle) -> Result<(), Error> {
    let db = couch.database("core", true);
    let manager = ResourceManager::new(logger.new(slog::o!("manager" => "users")), db, "users");
    manager.init().await?;
    let mut w = Watcher::<User>::start(logger.clone(), manager.clone(), arbiter);

    while let Some(res) = w.next().await {
        let event = match res {
            Ok(e) => e,
            Err(err) => {
                slog::error!(logger, "{}", err);
                continue;
            }
        };
        let res = on_user_changed(logger.clone(), &manager, event.resource).await;
        if let Err(err) = res {
            slog::error!(logger, "{}", err);
        }
    }

    Ok(())
}

async fn on_user_changed(logger: Logger, manager: &ResourceManager<User>, mut user: User) -> Result<(), Error> {
    let mut dirty = false;

    if user.metadata.is_just_created() {
        user.metadata.created_at = Some(Utc::now());
        dirty = true;
    }

    if user.metadata.is_deleted() {
        slog::info!(logger, "User {:?} was deleted!", user);
        return Ok(());
    }

    let enabled = user.spec.enabled;
    dirty = user.status.as_ref().map_or(dirty, |status| status.enabled != enabled);
    if dirty {
        user.status = Some(UserStatus { enabled });
        let name = user.metadata.name.clone();
        let user = manager.put(&name, user).await?;
        slog::info!(logger, "User {:?} was updated!", user);
    }
    Ok(())
}
