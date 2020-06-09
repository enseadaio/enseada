use actix_web::{HttpRequest, HttpResponse, Responder};

use crate::templates::ReDoc;

const SPEC: &str = include_str!(concat!(env!("OUT_DIR"), "/openapi.yml"));

pub async fn open_api() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/yaml")
        .body(SPEC)
}

pub async fn redoc(req: HttpRequest) -> impl Responder {
    let spec_url = req.url_for_static("open_api_spec").unwrap();
    ReDoc { spec_url: spec_url.to_string() }
}