use serde::{Deserialize, Serialize};

use enseada::couchdb::repository::Entity;
use enseada::guid::Guid;
use enseada::secure;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Repo {
    #[serde(rename = "_id")]
    id: Guid,
    #[serde(rename = "_rev", skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    group_id: String,
    artifact_id: String,
    decoded_location: String,
    public: bool,
}

impl Repo {
    pub fn new<G: ToString, A: ToString>(group_id: G, artifact_id: A, public: bool) -> Self {
        let group_id = group_id.to_string();
        let artifact_id = artifact_id.to_string();
        let location = Self::build_id(&group_id, &artifact_id);
        let decoded_location =
            String::from_utf8_lossy(&secure::base64::decode(&location).unwrap()).to_string();
        Self {
            id: Self::build_guid(&location),
            rev: None,
            group_id,
            artifact_id,
            decoded_location,
            public,
        }
    }

    pub fn group_id(&self) -> &str {
        &self.group_id
    }

    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }

    pub fn location(&self) -> &str {
        &self.decoded_location
    }

    pub fn is_public(&self) -> bool {
        self.public
    }
    
    #[inline]
    pub fn is_private(&self) -> bool {
        !self.is_public()
    }

    pub fn build_id<G: ToString, A: ToString>(group_id: G, artifact_id: A) -> String {
        let location = format!(
            "{}/{}",
            group_id.to_string().replace('.', "/"),
            artifact_id.to_string()
        );
        secure::base64::encode(location)
    }
}

impl Entity for Repo {
    fn build_guid(location: &str) -> Guid {
        Guid::partitioned("maven_repo", location)
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_composes_the_location() {
        let repo = Repo::new("io.enseada.test", "test-repo", true);
        let id = repo.id();
        let location = repo.location();
        let loc_b64 = secure::base64::encode(location);

        assert_eq!(format!("maven_repo:{}", loc_b64), id.to_string());
        assert_eq!("io/enseada/test/test-repo", location);
    }
}
