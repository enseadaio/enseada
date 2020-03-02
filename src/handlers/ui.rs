use actix_web::Responder;

use crate::templates::Index;

pub async fn index() -> impl Responder {
    Index { name: Some("Matteo") }
}