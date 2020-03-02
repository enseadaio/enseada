use actix_web::web;

use actix_files as fs;

use crate::handlers::health;
use crate::handlers::ui;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg
        .service(fs::Files::new("/dist", ".").show_files_listing())
        .service(
            web::scope("/ui")
                .route("", web::get().to(ui::index))
        )
        .route("/health", web::get().to(health::get));
}