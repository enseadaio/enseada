use serde::{Deserialize, Serialize};

use enseada::guid::Guid;

use crate::couchdb::repository::Entity;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Repo {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    group: String,
    name: String,
    tags: Vec<String>,
}

impl Repo {
    pub fn build_id(group: &str, name: &str) -> String {
        format!("{}-{}", group, name)
    }

    pub fn new(group: &str, name: &str) -> Self {
        let id = Self::build_guid(&Self::build_id(group, name));
        Self {
            id,
            rev: None,
            group: group.to_string(),
            name: name.to_string(),
            tags: Vec::new(),
        }
    }

    pub fn group(&self) -> &str {
        &self.group
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn full_name(&self) -> String {
        format!("{}/{}", &self.group, &self.name)
    }

    pub fn tags(&self) -> &Vec<String> {
        &self.tags
    }
}

impl Entity for Repo {
    fn build_guid(id: &str) -> Guid {
        Guid::partitioned("oci_repo", id)
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
