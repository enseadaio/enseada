use async_trait::async_trait;
use couchdb::db::Database;
use enseada::error::Error;
use enseada::secure;

use crate::couchdb::repository::Repository;
use crate::user::User;

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
        log::debug!("Authenticating user {}", username);
        let user = match self.find(username).await? {
            Some(user) => user,
            None => return Err(Error::from("authentication failed")),
        };

        if secure::verify_password(user.password_hash(), password)? {
            Ok(user)
        } else {
            Err(Error::from("authentication failed"))
        }
    }
}
