use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Tls {
    pub cert: PathBuf,
    pub key: PathBuf,
}
