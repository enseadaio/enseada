use actix_web::get;
use actix_web::web::ServiceConfig;
use actix_web::web::{Data, Json};
use serde::Serialize;

use enseada::couchdb::Couch;
use observability::{Status, StatusService};

use crate::couchdb;
use crate::http::error::ApiError;
use crate::http::error::ApiError::ServiceUnavailable;
use crate::http::responses;

pub fn mount(couch: Couch) -> Box<impl FnOnce(&mut ServiceConfig)> {
    Box::new(|cfg: &mut ServiceConfig| {
        let status = StatusService::new(couch);
        cfg.data(status);

        cfg.service(get);
    })
}

#[derive(Debug, Serialize, PartialEq)]
pub struct HealthResponse {
    pub status: String,
}

#[get("/health")]
pub async fn get(svc: Data<StatusService>) -> Result<Json<HealthResponse>, ApiError> {
    match svc.status().await {
        Status::Healty => responses::ok(HealthResponse {
            status: "ok".to_string(),
        }),
        Status::Unhealthy(err) => {
            log::error!("{}", &err);
            Err(ServiceUnavailable(err))
        }
    }
}
