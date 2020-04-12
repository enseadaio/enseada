use serde::{Deserialize, Serialize};
use crate::couchdb::guid::Guid;
use crate::couchdb::error::Error as CouchError;
use crate::error::Error;
use crate::secure;
use crate::couchdb::db::Database;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    password_hash: String,
}

impl User {
    pub fn build_guid(username: String) -> Guid {
        Guid::from(format!("user:{}", &username))
    }

    pub fn new(username: String, password: String) -> Result<User, Error> {
        let password_hash = secure::hash_password(password.as_str())?;
        let id = Self::build_guid(username);
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

pub struct UserService {
    db: Database
}

impl UserService {
    pub fn new(db: Database) -> UserService {
        UserService { db }
    }

    pub async fn find_user(&self, username: &str) -> Result<Option<User>, Error> {
        let guid = User::build_guid(username.to_string()).to_string();
        match self.db.get(guid.as_str()).await {
            Ok(user) => Ok(Some(user)),
            Err(err) => match err {
                CouchError::NotFound(_) => Ok(None),
                _ => Err(Error::from(err)),
            },
        }
    }

    pub async fn save_user(&self, user: User) -> Result<User, CouchError> {
        let guid = user.id().to_string();
        let res = self.db.put(guid.as_str(), &user).await?;
        let mut user = user;
        user.set_rev(res.rev);
        Ok(user)
    }

    pub async fn authenticate_user(&self, username: String, password: String) -> Result<User, Error> {
        log::debug!("Authenticating user {}", &username);
        let user = match self.find_user(&username).await? {
            Some(user) => user,
            None => return Err(Error::from("authentication failed")),
        };

        if secure::verify_password(&user.password_hash, &password)? {
            Ok(user)
        } else {
            Err(Error::from("authentication failed"))
        }
    }
}