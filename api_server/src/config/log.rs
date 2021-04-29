use serde::{Deserialize, Serialize};
use std::str::FromStr;
use config::{Config, ConfigError};
use slog::Level;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Log {
    level: String,
    format: LogFormat,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Text,
    Json
}

impl Log {
    pub fn set_defaults(cfg: &mut Config) -> Result<(), ConfigError> {
        cfg.set_default("log.level", "info")?;
        cfg.set_default("log.format", "json")?;
        Ok(())
    }

    pub fn level(&self) -> Level {
        let lvl = self.level.to_lowercase();
        Level::from_str(&lvl).expect(&format!("invalid log level '{}'", lvl))
    }
    pub fn format(&self) -> &LogFormat {
        &self.format
    }
}
