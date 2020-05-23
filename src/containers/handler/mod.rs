use actix_web::HttpResponse;
use actix_web::web::{Data, Path};
use serde::Deserialize;

use crate::containers::name::Name;
use crate::containers::storage::Provider;

pub mod manifest;
pub mod upload;

#[derive(Debug, Deserialize)]
pub struct NameParams {
    pub group: String,
    pub name: String,
}

pub async fn check_version() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub async fn test(
    path: Path<NameParams>,
) -> HttpResponse {
    let name = Name::from(path.into_inner());
    HttpResponse::Ok().body(format!("hello from Docker image {}/{}!", name.group(), name.name()))
}