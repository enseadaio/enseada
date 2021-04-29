use std::net::SocketAddr;

use config::{Config, ConfigError};
use serde::{Deserialize, Serialize};

use crate::config::tls::Tls;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Http {
    host: String,
    port: u16,
    tls: Option<Tls>,
}

impl Http {
    pub fn set_defaults(cfg: &mut Config) -> Result<(), ConfigError> {
        cfg.set_default("http.host", "[::]")?;
        cfg.set_default("http.port", 9623)?;
        Ok(())
    }

    pub fn address(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port).parse().unwrap()
    }

    pub fn tls(&self) -> Option<&Tls> {
        self.tls.as_ref()
    }
}
