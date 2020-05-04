use std::fmt;
use std::fmt::Debug;

use http::StatusCode;
use serde::{Deserialize, Serialize};
use serde::export::Formatter;

use crate::couchdb::db::Database;
use crate::couchdb::error::Error as CouchError;
use crate::couchdb::guid::Guid;
use crate::error::Error;
use crate::pagination::Page;
use crate::secure;

#[derive(Clone, Deserialize, Serialize)]
pub struct User {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    password_hash: String,
}

impl User {
    pub fn build_guid(username: &str) -> Guid {
        Guid::from(format!("user:{}", username))
    }

    pub fn new(username: String, password: String) -> Result<User, Error> {
        let password_hash = secure::hash_password(password.as_str())?;
        let id = Self::build_guid(&username);
        Ok(User { id, rev: None, password_hash })
    }

    pub fn id(&self) -> &Guid {
        &self.id
    }

    pub fn rev(&self) -> Option<String> {
        self.rev.clone()
    }

    pub fn username(&self) -> &String {
        self.id.id()
    }

    pub fn set_rev(&mut self, rev: String) -> &mut Self {
        self.rev = Some(rev);
        self
    }
}

impl Debug for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "User {{ id: {:?}, rev: {:?} }}", &self.id, &self.rev)
    }
}

pub struct UserService {
    db: Database
}

impl UserService {
    pub fn new(db: Database) -> UserService {
        UserService { db }
    }

    pub async fn list_users(&self, limit: usize, offset: usize) -> Result<Page<User>, Error> {
        let page = self.db.list("user", limit, offset).await?;
        Ok(Page::from(page))
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