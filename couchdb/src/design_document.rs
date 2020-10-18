use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Deserialize, Serialize)]
pub struct DesignDocument {
    #[serde(rename = "_id")]
    id: String,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    language: String,
    options: HashMap<String, serde_json::Value>,
    views: HashMap<String, ViewDoc>,
}

impl DesignDocument {
    pub fn new<N: Display>(name: N, partitioned: bool) -> Self {
        let mut options = HashMap::with_capacity(1);
        options.insert(
            "partitioned".to_string(),
            serde_json::to_value(&partitioned).unwrap(),
        );

        Self {
            id: format!("_design/{}", name),
            rev: None,
            language: "javascript".to_string(),
            options,
            views: HashMap::new(),
        }
    }

    pub fn add_view(&mut self, view: ViewDoc) -> &mut Self {
        self.views.insert(view.name.clone(), view);
        self
    }

    pub fn language(&self) -> &str {
        &self.language
    }
    pub fn options(&self) -> &HashMap<String, serde_json::Value> {
        &self.options
    }
    pub fn views(&self) -> &HashMap<String, ViewDoc> {
        &self.views
    }
}

#[derive(Deserialize, Serialize)]
pub struct ViewDoc {
    #[serde(skip)]
    name: String,
    map: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reduce: Option<String>,
}

impl ViewDoc {
    pub fn from_map_reduce<N: ToString, M: ToString, R: ToString>(
        name: N,
        map_fun: M,
        reduce_fun: R,
    ) -> Self {
        Self {
            name: name.to_string(),
            map: map_fun.to_string(),
            reduce: Some(reduce_fun.to_string()),
        }
    }

    pub fn from_map<N: ToString, M: ToString>(name: N, map_fun: M) -> Self {
        Self {
            name: name.to_string(),
            map: map_fun.to_string(),
            reduce: None,
        }
    }

    pub fn map(&self) -> &str {
        &self.map
    }
    pub fn reduce(&self) -> Option<&str> {
        self.reduce.as_deref()
    }
}
