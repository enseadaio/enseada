use structopt::StructOpt;
use crate::config::LogFormat;
use std::path::PathBuf;
use config::{Config, ConfigError};

#[derive(Debug, StructOpt)]
#[structopt(name = "enseada-api-server", about = "Enseada server")]
pub struct Opts {
    #[structopt(long, env = "ENSEADA_LOG_LEVEL")]
    pub log_level: Option<String>,
    #[structopt(long, env = "ENSEADA_LOG_FORMAT")]
    pub log_format: Option<String>,

    #[structopt(long, env = "ENSEADA_GRPC_HOST")]
    pub grpc_host: Option<String>,
    #[structopt(long, env = "ENSEADA_GRPC_PORT")]
    pub grpc_port: Option<i16>,
    #[structopt(long, env = "ENSEADA_GRPC_CA_CERT_FILE")]
    pub grpc_tls_ca_cert: Option<PathBuf>,
    #[structopt(long, env = "ENSEADA_GRPC_CERT_FILE")]
    pub grpc_tls_cert: Option<PathBuf>,
    #[structopt(long, env = "ENSEADA_GRPC_KEY_FILE")]
    pub grpc_tls_key: Option<PathBuf>,
}

impl Opts {
    pub fn set_cfg(self, cfg: &mut Config) -> Result<(), ConfigError> {
        if let Some(log_level) = self.log_level {
            cfg.set("log.level", log_level)?;
        }

        if let Some(log_format) = self.log_format {
            cfg.set("log.format", log_format)?;
        }

        if let Some(grpc_host) = self.grpc_host {
            cfg.set("grpc.host", grpc_host)?;
        }
        if let Some(grpc_port) = self.grpc_port {
            cfg.set("grpc.port", grpc_port.to_string())?;
        }
        if let Some(grpc_tls_ca_cert) = self.grpc_tls_ca_cert {
            cfg.set("grpc.tls.ca_cert", grpc_tls_ca_cert.to_str().unwrap())?;
        }
        if let Some(grpc_tls_cert) = self.grpc_tls_cert {
            cfg.set("grpc.tls.cert", grpc_tls_cert.to_str().unwrap())?;
        }
        if let Some(grpc_tls_key) = self.grpc_tls_key {
            cfg.set("grpc.tls.key", grpc_tls_key.to_str().unwrap())?;
        }

        Ok(())
    }
}
