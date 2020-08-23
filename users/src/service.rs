use async_trait::async_trait;

use enseada::couchdb::db::Database;
use enseada::couchdb::repository::Repository;
use enseada::error::Error;
use enseada::secure;

use crate::User;

#[derive(Debug)]
pub struct UserService {
    db: Database,
}

#[async_trait]
impl Repository<User> for UserService {
    fn db(&self) -> &Database {
        &self.db
    }
}

impl UserService {
    pub fn new(db: Database) -> UserService {
        UserService { db }
    }

    #[tracing::instrument]
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
