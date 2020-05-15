use regex::Regex;

pub use routes::routes;

use crate::docker::error::Error;

pub mod error;
pub mod handler;
pub mod header;
pub mod manifest;
pub mod mime;
mod routes;

pub type Result<T> = std::result::Result<T, Error>;

lazy_static! {
    pub static ref REGEX: Regex = Regex::new("[a-z0-9]+(?:[._-][a-z0-9]+)*").unwrap();
}

pub fn validate_name(group: &str, name: &str) -> bool {
    REGEX.is_match(group) && REGEX.is_match(name)
}

pub struct Name {
    group: String,
    name: String,
}

impl Name {
    pub fn new(group: String, name: String) -> Self {
        Name {
            group,
            name,
        }
    }

    pub fn group(&self) -> &str {
        &self.group
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}