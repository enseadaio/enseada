use crate::config::CONFIG;
use actix_web::web;

use crate::couchdb::client::Client;

pub mod client;
pub mod status;

pub fn add_couch_client(app: &mut web::ServiceConfig) {
    let couch = CONFIG.couchdb();
    let url = couch.url();
    let username = couch.username();
    let password = couch.password();
    let client = Client::new(url, username, password);
    app.data(client);
}