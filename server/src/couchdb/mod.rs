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
    pub const OCI: &str = "oci";
}

lazy_static! {
    pub static ref SINGLETON: Couch = from_global_config();
}

pub fn from_global_config() -> Couch {
    let couch = CONFIG.couchdb();
    let url = couch.url();
    let username = couch.username();
    let password = couch.password();
    Couch::new(url, username, password)
}
