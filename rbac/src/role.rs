use serde::{Deserialize, Serialize};

use enseada::couchdb::repository::Entity;
use enseada::guid::Guid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Role {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
}

impl Role {
    pub fn new(name: &str) -> Self {
        Role {
            id: Self::build_guid(name),
            rev: None,
        }
    }

    pub fn name(&self) -> &str {
        self.id.id()
    }
}

impl Entity for Role {
    fn build_guid(id: &str) -> Guid {
        Guid::from(format!("role:{}", id))
    }

    fn id(&self) -> &Guid {
        &self.id
    }

    fn rev(&self) -> Option<&str> {
        self.rev.as_deref()
    }

    fn set_rev(&mut self, rev: String) -> &mut Self {
        self.rev = Some(rev);
        self
    }
}
