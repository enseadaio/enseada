use std::sync::Arc;

use actix_web::web;

pub use migrate::migrate;

use crate::config::CONFIG;
use crate::couchdb::client::Client;
use crate::couchdb::db::Database;
use crate::couchdb::error::Error;
use crate::couchdb::status::Status;

pub mod client;
pub mod db;
pub mod error;
pub mod index;
mod migrate;
pub mod responses;
pub mod status;

lazy_static! {
    pub static ref SINGLETON: Couch = Couch::from_global_config();
}

type Result<T> = std::result::Result<T, Error>;

pub struct Couch {
    client: Arc<Client>,
}

impl Couch {
    pub fn from_global_config() -> Couch {
        let couch = CONFIG.couchdb();
        let url = couch.url();
        let username = couch.username();
        let password = couch.password();
        let client = Arc::new(Client::new(url, username, password));
        Couch { client }
    }

    pub fn database(&self, name: &str, partitioned: bool) -> Database {
        Database::new(self.client.clone(), name.to_string(), partitioned)
    }

    pub async fn status(&self) -> reqwest::Result<Status> {
        self.client.get("/_up", None::<bool>).await
    }
}

pub fn add_couch_client(app: &mut web::ServiceConfig) {
    app.data(&SINGLETON);
}
