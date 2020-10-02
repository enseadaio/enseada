use std::fmt::Debug;
use std::vec::IntoIter;

use serde::{Deserialize, Serialize};

use couchdb::responses::{FindResponse, RowsResponse};

#[derive(Debug, Deserialize, Serialize)]
pub struct Page<T> {
    count: usize,
    total: usize,
    offset: usize,
    limit: usize,
    items: Vec<T>,
}

impl<T> Page<T> {
    pub fn from_slice(items: Vec<T>, offset: usize, limit: usize, total: usize) -> Self {
        let count = items.len();
        Page {
            count,
            total,
            offset,
            limit,
            items,
        }
    }

    pub fn from_rows_response(
        res: RowsResponse<T>,
        limit: usize,
        offset: usize,
        total: usize,
    ) -> Self {
        if res.rows.len() <= limit {
            let items = res
                .rows
                .into_iter()
                .map(|raw| raw.doc.expect("RowResponse does not contain docs"))
                .collect();
            Page::from_slice(items, offset, limit, total)
        } else {
            let mut res = res;
            res.rows.remove(res.rows.len() - 1);
            let items = res
                .rows
                .into_iter()
                .map(|raw| raw.doc.expect("RowResponse does not contain docs"))
                .collect();
            Page::from_slice(items, offset, limit, total)
        }
    }

    pub fn from_find_response(
        res: FindResponse<T>,
        limit: usize,
        offset: usize,
        total: usize,
    ) -> Self {
        Self::from_slice(res.docs, offset, limit, total)
    }

    pub fn map<B, F>(self, f: F) -> Page<B>
    where
        F: FnMut(T) -> B,
    {
        Page {
            count: self.count,
            total: self.total,
            offset: self.offset,
            limit: self.limit,
            items: self.items.into_iter().map(f).collect(),
        }
    }

    pub fn is_first(&self) -> bool {
        self.offset == 0
    }

    pub fn is_last(&self) -> bool {
        let sum = self.offset + self.limit;
        sum >= self.total
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn total(&self) -> usize {
        self.total
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn limit(&self) -> usize {
        self.limit
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }
}

impl<T> IntoIterator for Page<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}
