use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
pub struct Tls {
    pub ca_cert: PathBuf,
    pub cert: PathBuf,
    pub key: PathBuf,
}
