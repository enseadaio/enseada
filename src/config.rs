use config::{Config, ConfigError, Environment};
use url::Url;

#[derive(Debug, Deserialize)]
pub struct Logging {
    level: String,
    format: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CouchDB {
    url: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
    log: Logging,
    couchdb: CouchDB,
}

impl Configuration {
    pub fn new() -> Result<Self, ConfigError> {
        dotenv();

        let mut c = Config::new();

        c.merge(Environment::with_prefix("enseada").separator("_"))?;

        c.set_default("log.level", "info")?;
        c.set_default("couchdb.url", "http://localhost:5984")?;

        c.try_into()
    }

    pub fn log(&self) -> &Logging {
        &self.log
    }

    pub fn couchdb(&self) -> &CouchDB {
        &self.couchdb
    }
}

impl Logging {
    pub fn level(&self) -> String {
        self.level.clone()
    }

    pub fn format(&self) -> String {
        self.format.as_ref().unwrap_or(&"text".to_owned()).clone()
    }
}

impl CouchDB {
    pub fn url(&self) -> Url {
        let url = self.url.as_ref().expect("missing couchdb.url").as_str();
        Url::parse(url).expect("failed to parse CouchDB URL")
    }

    pub fn username(&self) -> String {
        self.username
            .as_ref()
            .expect("missing couchdb.username")
            .clone()
    }

    pub fn password(&self) -> String {
        self.password
            .as_ref()
            .expect("missing couchdb.password")
            .clone()
    }
}

// Throw the Config struct into a CONFIG lazy_static to avoid multiple processing
lazy_static! {
    pub static ref CONFIG: Configuration = Configuration::new().expect("Configuration::new()");
}

#[cfg(debug_assertions)]
fn dotenv() {
    dotenv::dotenv().expect("dotenv::dotenv()");
}

#[cfg(not(debug_assertions))]
fn dotenv() {
    // noop
}
