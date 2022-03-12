use std::str::FromStr;

use serde::Deserialize;
use tracing::Level;
use tracing_subscriber::{EnvFilter, Registry};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::Config;

pub fn try_init(cfg: &Config) -> anyhow::Result<()> {
    let log_format = &cfg.log_format;

    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_from_env("ENSEADA_LOG"))
        .or_else(|_| EnvFilter::try_new(format!("enseada_server={}", cfg.log_level)))
        .unwrap();

    let json_logger = log_format.eq(&LogFormat::Json).then(|| {
        tracing_subscriber::fmt::layer()
            .json()
            .with_current_span(true)
            .flatten_event(true)
    });
    let text_logger = log_format
        .eq(&LogFormat::Text)
        .then(|| tracing_subscriber::fmt::layer().pretty());

    Registry::default()
        .with(env_filter)
        .with(json_logger)
        .with(text_logger)
        .try_init()?;

    Ok(())
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Text,
}

impl FromStr for LogFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "text" | "" => Ok(Self::Text),
            _ => Err(format!("Unsupported log format '{}'", s)),
        }
    }
}

pub type LogLevel = Level;
