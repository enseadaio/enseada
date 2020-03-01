use config::{Config, ConfigError, Environment};

use dotenv::dotenv;

#[derive(Debug, Deserialize)]
pub struct Logging {
    level: String,
    format: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
    log: Logging,
}

impl Configuration {
    pub fn new() -> Result<Self, ConfigError> {
        dotenv().ok();

        let mut c = Config::new();

        c.merge(Environment::with_prefix("enseada").separator("_"))?;

        c.set_default("log.level", "info")?;

        c.try_into()
    }

    pub fn log(&self) -> &Logging {
        &self.log
    }
}

impl Logging {
    pub fn level(&self) -> &String {
        &self.level
    }

    pub fn format(&self) -> &Option<String> {
        &self.format
    }
}