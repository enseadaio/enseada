use actix_web::web::{Data, Json};
use serde::{Serialize, Deserialize};

use crate::couchdb;
use crate::couchdb::status::Status;
use crate::error::ApiError;
use crate::error::ApiError::ServiceUnavailable;
use crate::responses;

#[derive(Debug, Serialize, PartialEq)]
pub struct HealthResponse {
    pub status: String,
}

pub async fn get(couch: Data<couchdb::Couch>) -> Result<Json<HealthResponse>, ApiError> {
    match couch.status().await {
        Ok(Status { status }) => responses::ok(HealthResponse { status }),
        Err(err) => {
            log::error!("{}", err);
            Err(ServiceUnavailable(
                "database connection refused".to_string(),
            ))
        }
    }
}
