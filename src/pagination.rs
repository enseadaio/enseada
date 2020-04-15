use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use crate::couchdb::responses::RowsResponse;

#[derive(Deserialize, Serialize, Debug)]
pub struct Page<T> {
    count: usize,
    total_count: usize,
    offset: usize,
    items: Vec<T>,
}

impl<T> Page<T> {
    pub fn map<B, F>(&self, f: F) -> Page<B>
        where
            F: FnMut(&T) -> B
    {
        Page {
            count: self.count,
            total_count: self.total_count,
            offset: self.offset,
            items: self.items.iter().map(f).collect(),
        }
    }
}

impl<T: Clone> From<RowsResponse<T>> for Page<T> {
    fn from(rows: RowsResponse<T>) -> Self {
        let items: Vec<T> = rows.rows.iter().map(|res| res.doc.clone()).collect();
        Page {
            count: items.len(),
            total_count: rows.total_rows,
            offset: rows.offset,
            items,
        }
    }
}