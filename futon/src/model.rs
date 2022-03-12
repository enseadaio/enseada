use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Info {
    couchdb: String,
    uuid: String,
    vendor: Vendor,
    version: String,
}

#[derive(Debug, Deserialize)]
pub struct Vendor {
    name: String,
    version: Option<String>,
}

pub enum Up {
    Ok,
    Unavailable,
}
