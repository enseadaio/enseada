use config::{Config, ConfigError};
use serde::Deserialize;

use std::time::Duration;

const DEFAULT_POLLING_INTERVAL: &str = "5 minutes";

#[derive(Clone, Debug, Deserialize)]
pub struct Controllers {
    users: Controller,
}

impl Controllers {
    pub fn set_defaults(cfg: &mut Config) -> Result<(), ConfigError> {
        Controller::set_defaults(cfg, "users")?;
        Ok(())
    }


    pub fn users(&self) -> &Controller {
        &self.users
    }

    pub fn controller(&self, name: &str) -> Option<&Controller> {
        match name {
            "users" => Some(&self.users),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Controller {
    #[serde(with = "humantime_serde")]
    polling_interval: Duration
}

impl Controller {
    fn set_defaults(cfg: &mut Config, name: &str) -> Result<(), ConfigError> {
        cfg.set_default(&format!("controllers.{}.polling_interval", name), DEFAULT_POLLING_INTERVAL)?;
        Ok(())
    }


    pub fn polling_interval(&self) -> Duration {
        self.polling_interval
    }
}

