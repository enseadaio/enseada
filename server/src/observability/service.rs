use couchdb::Couch;

use crate::observability::entity::Status;

pub struct StatusService {
    client: Couch,
}

impl StatusService {
    pub fn new(client: Couch) -> Self {
        Self { client }
    }

    pub async fn status(&self) -> Status {
        match self.client.status().await {
            Ok(_) => Status::Healty,
            Err(err) => Status::Unhealthy(err.to_string()),
        }
    }
}
