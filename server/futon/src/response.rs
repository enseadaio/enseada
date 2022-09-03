use hyper::StatusCode;
use serde::Deserialize;
use serde_json::Value;
use std::fmt::{Debug, Formatter};

#[derive(Deserialize)]
pub struct RowsResponse<T> {
    pub offset: usize,
    pub rows: Vec<Row<T>>,
    pub total_rows: usize,
    pub update_seq: Option<usize>,
}

impl<T> IntoIterator for RowsResponse<T> {
    type Item = Row<T>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows.into_iter()
    }
}

impl<T: Debug> Debug for RowsResponse<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RowsResponse")
            .field("offset", &self.offset)
            .field("rows", &self.rows)
            .field("total_rows", &self.total_rows)
            .field("update_seq", &self.update_seq)
            .finish()
    }
}

#[derive(Debug, Deserialize)]
pub struct Row<T> {
    pub id: String,
    pub key: String,
    pub value: Value,
    pub doc: Option<T>,
}

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

#[derive(Debug, Copy, Clone)]
pub enum Up {
    Ok,
    Unavailable,
}

impl From<Up> for StatusCode {
    fn from(up: Up) -> Self {
        match up {
            Up::Ok => StatusCode::OK,
            Up::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CouchErrorBody {
    pub error: String,
    pub reason: String,
}
