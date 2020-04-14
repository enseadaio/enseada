use actix_web::{HttpResponse, Responder, HttpRequest};
use crate::templates::ReDoc;

const SPEC: &str = include_str!(concat!(env!("OUT_DIR"), "/openapi.yml"));

pub async fn open_api() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/yaml")
        .body(SPEC)
}

pub async fn redoc() -> impl Responder {
    ReDoc { spec_url: "/api/docs/openapi.yml" }
}