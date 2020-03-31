use actix_web::web;


use crate::config::CONFIG;
use crate::couchdb::client::Client;
use crate::couchdb::db::Database;
use crate::couchdb::status::Status;

pub mod client;
pub mod db;
pub mod errors;
pub mod responses;
pub mod status;
pub mod guid;

pub struct Couch {
    client: Box<Client>,
}

impl Couch {
    pub fn database(&self, name: &str) -> Database {
        Database::new(self.client.clone(), name.to_owned())
    }

    pub async fn status(&self) -> reqwest::Result<Status> {
        self.client.get("/_up").await
    }
}

pub fn add_couch_client(app: &mut web::ServiceConfig) {
    let couch = CONFIG.couchdb();
    let url = couch.url();
    let username = couch.username();
    let password = couch.password();
    let client = Box::new(Client::new(url, username, password));
    let couch = Couch { client };
    app.data(couch);
}
