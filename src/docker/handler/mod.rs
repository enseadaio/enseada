use actix_web::HttpResponse;
use actix_web::web::Path;
use serde::Deserialize;
use crate::docker::header;

pub mod manifest;

pub async fn check_version() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[derive(Debug, Deserialize)]
pub struct ImageNamePath {
    group: String,
    name: String,
}

pub async fn test(
    path: Path<ImageNamePath>
) -> HttpResponse {
    HttpResponse::Ok().body(format!("hello from Docker image {}/{}!", &path.group, &path.name))
}