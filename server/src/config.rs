use config::{Config, ConfigError, Environment};
use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    port: i16,
    log: Logging,
    couchdb: CouchDB,
    tls: TLS,
    public: Public,
    secret: Secret,
    root: Root,
    storage: Storage,
    oci: OCI,
}

#[derive(Debug, Deserialize)]
pub struct Logging {
    level: String,
    rootlevel: String,
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
struct WithOptionalPath {
    path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Public {
    host: Url,
}

#[derive(Debug, Deserialize)]
struct Secret {
    key: String,
}

#[derive(Debug, Deserialize)]
struct Root {
    password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "provider")]
pub enum Storage {
    S3 {
        bucket: String,
        endpoint: Option<String>,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct OCI {
    subdomain: String,
}


impl Configuration {
    pub fn new() -> Result<Self, ConfigError> {
        dotenv();

        let mut c = Config::new();

        c.merge(Environment::with_prefix("enseada").separator("_"))?;

        // Defaults
        c.set_default("port", 9623)?;
        c.set_default("tls.enabled", false)?;
        c.set_default("tls.cert.path", None::<String>)?;
        c.set_default("tls.key.path", None::<String>)?;

        let port = c.get_int("port")?;
        let proto = if c.get_bool("tls.enabled")? { "https" } else { "http" };
        c.set_default("public.host", format!("{}://localhost:{}", proto, port))?;

        c.set_default("log.level", "info")?;
        c.set_default("log.rootlevel", "warn")?;
        c.set_default("couchdb.url", "http://localhost:5984")?;
        c.set_default("oci.subdomain", "containers")?;


        // Validations
        let secret_key = c.get_str("secret.key")?;
        if secret_key.len() < 32 {
            return Err(ConfigError::Message("insecure secret key, must be at least 32 bytes".to_string()));
        }

        let root_pwd = c.get_str("root.password")?;
        if root_pwd.len() < 8 {
            return Err(ConfigError::Message("insecure root password, must be at least 8 characters".to_string()));
        }

        // Deserialize
        c.try_into()
    }

    pub fn port(&self) -> i16 {
        self.port
    }

    pub fn public_host(&self) -> &Url {
        &self.public.host
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

    pub fn secret_key(&self) -> String {
        self.secret.key.clone()
    }

    pub fn root_password(&self) -> String {
        self.root.password.clone()
    }

    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    pub fn oci(&self) -> &OCI {
        &self.oci
    }
}

impl Logging {
    pub fn level(&self) -> String {
        self.level.clone()
    }

    pub fn root_level(&self) -> String {
        self.rootlevel.clone()
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

impl OCI {
    pub fn subdomain(&self) -> String {
        self.subdomain.clone()
    }
}

// Throw the Config struct into a CONFIG lazy_static to avoid multiple processing
lazy_static! {
    pub static ref CONFIG: Configuration = Configuration::new().expect("failed to load configuration");
}

#[cfg(debug_assertions)]
fn dotenv() {
    dotenv::dotenv().expect("dotenv::dotenv()");
}

#[cfg(not(debug_assertions))]
fn dotenv() {
    // noop
}
