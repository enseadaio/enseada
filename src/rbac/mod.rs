use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::couchdb::db::Database;
use crate::couchdb::guid::Guid;

mod model;

pub struct Enforcer {
    db: Database,
}

impl Enforcer {
    pub fn new(db: Database) -> Self {
        Enforcer { db }
    }

    pub async fn load_rules(&mut self) {
        self.db.list("rule")
    }
}

#[derive(Serialize, Deserialize)]
pub struct Rule {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    sub: String,
    obj: String,
    act: String,
}

impl Rule {
    pub fn build_guid(id: &str) -> Guid {
        Guid::from(format!("rule:{}", id))
    }

    pub fn new(sub: String, obj: String, act: String) -> Self {
        let uuid = Uuid::new_v4().to_string();
        let id = Self::build_guid(&uuid);
        Rule { id, rev: None, sub, obj, act }
    }
}