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
pub struct TLS {
    enabled: bool,
    cert: WithOptionalPath,
    key: WithOptionalPath,
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
    log: Logging,
    couchdb: CouchDB,
    tls: TLS,
    public: Public,
    port: i16,
}

#[derive(Debug, Deserialize)]
struct WithOptionalPath {
    path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Public {
    host: Option<String>,
}


impl Configuration {
    pub fn new() -> Result<Self, ConfigError> {
        dotenv();

        let mut c = Config::new();

        c.merge(Environment::with_prefix("enseada").separator("_"))?;

        c.set_default("port", 9623)?;
        c.set_default("public.host", "localhost")?;
        c.set_default("log.level", "info")?;
        c.set_default("couchdb.url", "http://localhost:5984")?;
        c.set_default("tls.enabled", false)?;
        // So we don't have 'missing field' errors
        c.set_default("tls.cert.path", None::<String>)?;
        c.set_default("tls.key.path", None::<String>)?;

        c.try_into()
    }

    pub fn port(&self) -> i16 {
        self.port
    }

    pub fn public_host(&self) -> Option<String> {
        self.public.host.clone()
    }

    pub fn log(&self) -> &Logging {
        &self.log
    }

    pub fn couchdb(&self) -> &CouchDB {
        &self.couchdb
    }

    pub fn tls(&self) -> &TLS {
        &self.tls
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

impl TLS {
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn cert_path(&self) -> Option<String> {
        self.cert.path.clone()
    }

    pub fn key_path(&self) -> Option<String> {
        self.key.path.clone()
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
