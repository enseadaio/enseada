use couchdb::Couch;
pub use migrate::migrate;

use crate::config::Configuration;

mod migrate;

pub mod name {
    pub const OAUTH: &str = "oauth";
    pub const USERS: &str = "users";
    pub const RBAC: &str = "acl";
    pub const OCI: &str = "oci";
    pub const MAVEN: &str = "maven";
}

pub fn from_config(cfg: &Configuration) -> Couch {
    let couch = cfg.couchdb();
    let url = couch.url();
    let username = couch.username();
    let password = couch.password();
    Couch::new(url, username, password)
}
