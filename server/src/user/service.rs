use std::pin::Pin;
use std::sync::Arc;

use actix::prelude::*;

use couchdb::db::Database;
use enseada::error::Error;
use enseada::pagination::{Cursor, Page};
use enseada::secure;

use crate::user::User;

pub struct UserService {
    db: Arc<Database>,
}

impl UserService {
    pub fn new(db: Database) -> UserService {
        UserService { db: Arc::new(db) }
    }

    pub async fn list_users(
        &self,
        limit: usize,
        cursor: Option<&Cursor>,
    ) -> Result<Page<User>, Error> {
    }

    pub async fn find_user(&self, username: &str) -> Result<Option<User>, Error> {
        let guid = User::build_guid(username).to_string();
        self.db.get(guid.as_str()).await.map_err(Error::from)
    }

    pub async fn save_user(&self, user: User) -> Result<User, Error> {
        let guid = user.id().to_string();
        let res = self.db.put(guid.as_str(), &user).await?;
        let mut user = user;
        user.set_rev(res.rev);
        Ok(user)
    }

    pub async fn delete_user(&self, user: &User) -> Result<(), Error> {
        let id = user.id().to_string();
        let rev = match user.rev() {
            Some(rev) => rev,
            None => panic!("user {} is missing rev", id),
        };
        self.db.delete(&id, &rev).await.map_err(Error::from)
    }

    pub async fn authenticate_user(&self, username: &str, password: &str) -> Result<User, Error> {
        log::debug!("Authenticating user {}", username);
        let user = match self.find_user(username).await? {
            Some(user) => user,
            None => return Err(Error::from("authentication failed")),
        };

        if secure::verify_password(&user.password_hash, password)? {
            Ok(user)
        } else {
            Err(Error::from("authentication failed"))
        }
    }
}

impl Default for UserService {
    fn default() -> Self {
        let couch = &crate::couchdb::SINGLETON;
        let db = couch.database(crate::couchdb::name::USERS, true);
        UserService::new(db)
    }
}

impl Actor for UserService {
    type Context = Context<Self>;
}

impl actix::Supervised for UserService {}

impl ArbiterService for UserService {
    fn service_started(&mut self, _ctx: &mut Context<Self>) {
        log::info!("Started user subsystem")
    }
}

#[derive(Message)]
#[rtype(result = "Result<Page<User>, Error>")]
pub struct ListUsers {
    pub limit: usize,
    pub cursor: Option<Cursor>,
}

impl Handler<ListUsers> for UserService {
    type Result = Pin<Box<dyn Future<Output = Result<Page<User>, Error>>>>;

    fn handle(&mut self, msg: ListUsers, ctx: &mut Self::Context) -> Self::Result {
        let fut = self.list_users(msg.limit, msg.cursor.as_ref());
        Box::pin(async move { fut.await })
    }
}
