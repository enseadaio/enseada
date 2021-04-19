use config::{Config, ConfigError};
use serde::Deserialize;

use crate::config::tls::Tls;
use std::net::SocketAddr;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Grpc {
    host: String,
    port: u16,
    tls: Option<Tls>,
}

impl Grpc {
    pub fn set_defaults(cfg: &mut Config) -> Result<(), ConfigError> {
        cfg.set_default("grpc.host", "[::]")?;
        cfg.set_default("grpc.port", 9624)?;
        Ok(())
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn address(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port).parse().unwrap()
    }

    pub fn tls(&self) -> &Option<Tls> {
        &self.tls
    }
}
