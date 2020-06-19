use std::fmt;
use std::fmt::Display;

use serde::export::Formatter;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use couchdb::responses::{FindResponse, RowsResponse};

use crate::error::Error;

#[derive(Deserialize, Serialize, Debug)]
pub struct Page<T> {
    count: usize,
    next_cursor: Option<Cursor>,
    items: Vec<T>,
}

impl<T: Clone> Page<T> {
    pub fn from_slice(items: Vec<T>, next_cursor: Option<Cursor>) -> Self {
        let count = items.len();
        Page {
            count,
            next_cursor,
            items,
        }
    }

    pub fn from_rows_response(res: RowsResponse<T>, limit: usize) -> Self {
        if res.rows.len() <= limit {
            let items = res.rows.iter().map(|raw| raw.doc.clone()).collect();
            Page::from_slice(items, None)
        } else {
            let mut res = res;
            let last = res.rows.remove(res.rows.len() - 1);
            let items = res.rows.iter().map(|raw| raw.doc.clone()).collect();
            Page::from_slice(
                items,
                Some(Cursor::b64_encoded(
                    serde_json::to_string(&last.key).unwrap(),
                )),
            )
        }
    }

    pub fn from_find_response(res: FindResponse<T>, limit: usize) -> Self {
        let bookmark = if res.docs.len() < limit {
            None
        } else {
            Some(Cursor::b64_encoded(res.bookmark))
        };

        Self::from_slice(res.docs, bookmark)
    }

    pub fn map<B, F>(self, f: F) -> Page<B>
    where
        F: FnMut(&T) -> B,
    {
        Page {
            count: self.count,
            next_cursor: self.next_cursor,
            items: self.items.iter().map(f).collect(),
        }
    }
}

#[derive(Debug)]
pub struct Cursor(String);

impl Cursor {
    pub fn from_b64<T: AsRef<[u8]>>(input: T) -> Result<Self, Error> {
        let decoded = base64::decode(input)?;
        Ok(Cursor(String::from_utf8(decoded)?))
    }

    pub fn b64_encoded<T: AsRef<[u8]>>(input: T) -> Self {
        let encoded = base64::encode(input);
        Cursor(encoded)
    }
}

impl Display for Cursor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for Cursor {
    fn from(s: String) -> Self {
        Cursor(s)
    }
}

impl Serialize for Cursor {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Cursor {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Self::from)
    }
}
