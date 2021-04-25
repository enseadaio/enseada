use structopt::StructOpt;
use crate::config::LogFormat;
use std::path::PathBuf;
use config::{Config, ConfigError};
use url::Url;

#[derive(Debug, StructOpt)]
#[structopt(name = "enseada-api-server", about = "Enseada server")]
pub struct Opts {
    #[structopt(long, env = "ENSEADA_LOG_LEVEL")]
    pub log_level: Option<String>,
    #[structopt(long, env = "ENSEADA_LOG_FORMAT")]
    pub log_format: Option<String>,

    #[structopt(long, env = "ENSEADA_HTTP_HOST")]
    pub http_host: Option<String>,
    #[structopt(long, env = "ENSEADA_HTTP_PORT")]
    pub http_port: Option<i16>,
    #[structopt(long, env = "ENSEADA_HTTP_CERT_FILE")]
    pub http_tls_cert: Option<PathBuf>,
    #[structopt(long, env = "ENSEADA_HTTP_KEY_FILE")]
    pub http_tls_key: Option<PathBuf>,

    #[structopt(long, env = "ENSEADA_COUCHDB_URL")]
    pub couchdb_url: Option<Url>,
    #[structopt(long, env = "ENSEADA_COUCHDB_USERNAME")]
    pub couchdb_username: Option<String>,
    #[structopt(long, env = "ENSEADA_COUCHDB_PASSWORD")]
    pub couchdb_password: Option<String>,
}

impl Opts {
    pub fn set_cfg(self, cfg: &mut Config) -> Result<(), ConfigError> {
        if let Some(log_level) = self.log_level {
            cfg.set("log.level", log_level)?;
        }

        if let Some(log_format) = self.log_format {
            cfg.set("log.format", log_format)?;
        }

        if let Some(http_host) = self.http_host {
            cfg.set("http.host", http_host)?;
        }
        if let Some(http_port) = self.http_port {
            cfg.set("http.port", http_port.to_string())?;
        }
        if let Some(http_tls_cert) = self.http_tls_cert {
            cfg.set("http.tls.cert", http_tls_cert.to_str().unwrap())?;
        }
        if let Some(http_tls_key) = self.http_tls_key {
            cfg.set("http.tls.key", http_tls_key.to_str().unwrap())?;
        }

        if let Some(couchdb_url) = self.couchdb_url {
            cfg.set("couchdb.url", couchdb_url.to_string())?;
        }

        if let Some(couchdb_username) = self.couchdb_username {
            cfg.set("couchdb.username", couchdb_username)?;
        }

        if let Some(couchdb_password) = self.couchdb_password {
            cfg.set("couchdb.password", couchdb_password)?;
        }

        Ok(())
    }
}
