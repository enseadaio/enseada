use std::time::Duration;

use config::{Config, ConfigError};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GarbageCollection {
    #[serde(with = "humantime_serde")]
    polling_interval: Duration,
}

impl GarbageCollection {
    pub fn set_defaults(cfg: &mut Config) -> Result<(), ConfigError> {
        cfg.set_default("gc.polling_interval", "30s")?;
        Ok(())
    }

    pub fn polling_interval(&self) -> Duration {
        self.polling_interval
    }
}
