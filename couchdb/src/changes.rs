use serde::Deserialize;

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

#[derive(Debug, Deserialize)]
pub struct Change {
    rev: String,
}