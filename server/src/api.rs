use axum::Router;

use futon::Couch;

pub fn routes(couch: &Couch) -> Router {
    Router::new().nest("/auth", auth::routes(couch))
}
