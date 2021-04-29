use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ChangeEvent {
    Next {
        id: String,
        seq: String,
        changes: Vec<Change>,
        deleted: Option<bool>,
    },
    End {
        last_seq: String,
        pending: u64,
    },
}

#[derive(Debug, Deserialize, Clone)]
pub struct Change {
    pub rev: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ChangeRequest {
    pub feed: String,
    pub since: String,
    pub filter: Option<String>,
}
