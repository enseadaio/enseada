use std::sync::Arc;

use actix_web::web;

use couchdb::Couch;
pub use migrate::migrate;

use crate::config::CONFIG;

mod migrate;
pub mod repository;

pub mod name {
    pub const OAUTH: &str = "oauth";
    pub const USERS: &str = "users";
    pub const RBAC: &str = "rbac";
}

lazy_static! {
    pub static ref SINGLETON: Couch = from_global_config();
}

fn from_global_config() -> Couch {
    let couch = CONFIG.couchdb();
    let url = couch.url();
    let username = couch.username();
    let password = couch.password();
    Couch::new(url, username, password)
}

pub fn add_couch_client(app: &mut web::ServiceConfig) {
    app.data(from_global_config());
}
