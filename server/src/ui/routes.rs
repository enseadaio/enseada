use actix_web::web::ServiceConfig;
use actix_web::{get, Responder};

use crate::templates::Index;

pub fn mount(cfg: &mut ServiceConfig) {
    cfg.service(index);
}

#[get("/ui")]
pub async fn index() -> impl Responder {
    Index { name: None }
}
