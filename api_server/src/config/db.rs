
use config::{Config, ConfigError};
use serde::Deserialize;

use url::Url;

#[derive(Clone, Debug, Deserialize)]
pub struct CouchDB {
    url: Url,
    username: String,
    password: String,
}

impl CouchDB {
    pub fn set_defaults(cfg: &mut Config) -> Result<(), ConfigError> {
        cfg.set_default("couchdb.url", "http://localhost:5984")?;
        cfg.set_default("couchdb.username", "")?;
        cfg.set_default("couchdb.password", "")?;
        Ok(())
    }

    pub fn url(&self) -> &Url {
        &self.url
    }


    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}
