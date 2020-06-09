use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Status {
    pub status: String,
}
