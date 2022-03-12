use std::net::SocketAddr;

use clap::Parser;

use crate::logger::{LogFormat, LogLevel};

#[derive(Debug, Parser)]
pub struct Config {
    #[clap(long, env = "ENSEADA_LOG_LEVEL", default_value = "info")]
    pub log_level: LogLevel,
    #[clap(long, env = "ENSEADA_LOG_FORMAT", default_value = "json")]
    pub log_format: LogFormat,

    #[clap(long, env = "ENSEADA_HTTP_HOST", default_value = "[::]")]
    pub http_host: String,
    #[clap(long, env = "ENSEADA_HTTP_PORT", default_value = "9623")]
    pub http_port: i16,

    #[clap(long, env = "ENSEADA_COUCHDB_URL")]
    pub couchdb_url: String,
}

impl Config {
    pub fn http_address(&self) -> SocketAddr {
        format!("{}:{}", self.http_host, self.http_port)
            .parse()
            .expect("invalid HTTP host or port")
    }
}
