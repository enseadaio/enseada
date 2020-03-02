use actix_web::web::{Json, Data};
use crate::couchdb;
use crate::errors::ApiError;
use crate::errors::ApiError::ServiceUnavailable;
use crate::responses;
use crate::couchdb::status::Status;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct HealthResponse {
    pub status: String,
}

pub async fn get_health(couch: Data<couchdb::client::Client>) -> Result<Json<HealthResponse>, ApiError> {
    match couch.status().await {
        Ok(Status { status }) => responses::ok(HealthResponse {
            status,
        }),
        Err(err) => {
            log::error!("{}", err);
            Err(ServiceUnavailable("database connection refused".to_string()))
        }
    }
}