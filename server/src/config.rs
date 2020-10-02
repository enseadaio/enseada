use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
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
    tracing: Tracing,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Logging {
    level: String,
    root_level: String,
    format: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CouchDB {
    url: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TLS {
    enabled: bool,
    cert: WithOptionalPath,
    key: WithOptionalPath,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct WithOptionalPath {
    path: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Public {
    host: Url,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Secret {
    key: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Root {
    password: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", tag = "provider")]
pub enum Storage {
    S3 {
        bucket: String,
        endpoint: Option<String>,
        access_key_id: Option<String>,
        secret_access_key: Option<String>,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OCI {
    host: String,
    max_body_size: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tracing {
    log: bool,
    level: String,
}

impl Configuration {
    pub fn new() -> Result<Self, ConfigError> {
        let mut c = Config::new();

        c.merge(File::with_name("enseada").required(false))?;
        c.merge(Environment::with_prefix("enseada").separator("__"))?;

        // Defaults
        c.set_default("port", 9623)?;
        c.set_default("tls.enabled", false)?;
        c.set_default("tls.cert.path", None::<String>)?;
        c.set_default("tls.key.path", None::<String>)?;

        let port = c.get_int("port")?;
        let proto = if c.get_bool("tls.enabled")? {
            "https"
        } else {
            "http"
        };
        c.set_default("public.host", format!("{}://localhost:{}", proto, port))?;

        c.set_default("log.level", "info")?;
        c.set_default("log.root_level", "warn")?;
        c.set_default("couchdb.url", "http://localhost:5984")?;
        c.set_default("oci.host", "containers.localhost")?;
        c.set_default("oci.max_body_size", 10_737_418_240)?; // 10 Gib
        c.set_default("tracing.log", false)?;
        c.set_default("tracing.level", "info")?;

        // Validations
        let secret_key = c.get_str("secret.key")?;
        if secret_key.len() < 32 {
            return Err(ConfigError::Message(
                "insecure secret key, must be at least 32 bytes".to_string(),
            ));
        }

        let root_pwd = c.get_str("root.password")?;
        if root_pwd.len() < 8 {
            return Err(ConfigError::Message(
                "insecure root password, must be at least 8 characters".to_string(),
            ));
        }

        // Deserialize, Serialize
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

    pub fn tracing(&self) -> &Tracing {
        &self.tracing
    }
}

impl Logging {
    pub fn level(&self) -> String {
        self.level.clone()
    }

    pub fn root_level(&self) -> String {
        self.root_level.clone()
    }

    pub fn format(&self) -> String {
        self.format
            .as_deref()
            .unwrap_or_else(|| "text")
            .to_lowercase()
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
    pub fn host(&self) -> String {
        self.host.clone()
    }

    pub fn max_body_size(&self) -> usize {
        self.max_body_size
    }
}

impl Tracing {
    pub fn log(&self) -> bool {
        self.log
    }

    pub fn level(&self) -> tracing::Level {
        match self.level.to_lowercase().as_str() {
            "error" => tracing::Level::ERROR,
            "warn" => tracing::Level::WARN,
            "info" => tracing::Level::INFO,
            "debug" => tracing::Level::DEBUG,
            "trace" => tracing::Level::TRACE,
            _ => tracing::Level::INFO,
        }
    }
}

// Throw the Config struct into a CONFIG lazy_static to avoid multiple processing
lazy_static! {
    pub static ref CONFIG: Configuration =
        Configuration::new().expect("failed to load configuration");
}
