use std::sync::{Arc, RwLock};

use async_trait::async_trait;

use enseada::couchdb::db::Database;
use enseada::couchdb::repository::Repository;
use enseada::error::Error;
use enseada::secure;
use events::EventBus;

use crate::events::{UserCreated, UserDeleted, UserUpdated};
use crate::User;

#[derive(Debug)]
pub struct UserService {
    db: Database,
    bus: Arc<RwLock<EventBus>>,
}

#[async_trait]
impl Repository<User> for UserService {
    fn db(&self) -> &Database {
        &self.db
    }

    async fn created(&self, entity: &User) {
        let event = UserCreated::from(entity);
        let bus = self.bus.read().expect("created() EventBus unlock");
        bus.broadcast(event);
    }

    async fn updated(&self, entity: &User) {
        let event = UserUpdated::from(entity);
        let bus = self.bus.read().expect("updated() EventBus unlock");
        bus.broadcast(event);
    }

    async fn deleted(&self, entity: &User) {
        let event = UserDeleted::from(entity);
        let bus = self.bus.read().expect("deleted() EventBus unlock");
        bus.broadcast(event);
    }
}

impl UserService {
    pub fn new(db: Database, bus: Arc<RwLock<EventBus>>) -> UserService {
        UserService { db, bus }
    }

    #[tracing::instrument(skip(password))]
    pub async fn authenticate_user(&self, username: &str, password: &str) -> Result<User, Error> {
        log::debug!("authenticating user {}", username);
        let user = match self.find(username).await? {
            Some(user) => user,
            None => {
                log::debug!("user {} not found", username);
                return Err(Error::from("authentication failed"));
            }
        };

        if !user.is_enabled() {
            log::debug!("user {} is disabled", username);
            return Err(Error::from("authentication failed"));
        }

        if secure::verify_password(user.password_hash(), password)? {
            Ok(user)
        } else {
            log::debug!("authentication failed for user {}", username);
            Err(Error::from("authentication failed"))
        }
    }
}
