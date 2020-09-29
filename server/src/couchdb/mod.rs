use couchdb::Couch;
pub use migrate::migrate;

use crate::config::{Configuration, CONFIG};

mod migrate;

pub mod name {
    pub const OAUTH: &str = "oauth";
    pub const USERS: &str = "users";
    pub const RBAC: &str = "rbac";
    pub const OCI: &str = "oci";
    pub const MAVEN: &str = "maven";
}

lazy_static! {
    pub static ref SINGLETON: Couch = from_global_config();
}

pub fn from_global_config() -> Couch {
    from_config(&CONFIG)
}

pub fn from_config(cfg: &Configuration) -> Couch {
    let couch = cfg.couchdb();
    let url = couch.url();
    let username = couch.username();
    let password = couch.password();
    Couch::new(url, username, password)
}
