use actix_files as fs;
use actix_web::{FromRequest, web};

use crate::http::handler::{health, oauth, ui, user};
use crate::oauth::request::{AuthorizationRequest, TokenRequest};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(fs::Files::new("/static", "./dist").show_files_listing())
        .route("/health", web::get().to(health::get))
        .service(web::scope("/ui")
            .route("", web::get().to(ui::index)))
        .service(web::scope("/oauth")
            .app_data(web::Query::<AuthorizationRequest>::configure(oauth::error::handle_query_errors))
            .app_data(web::Form::<TokenRequest>::configure(oauth::error::handle_form_errors))
            .route("/authorize", web::get().to(oauth::login_form))
            .route("/authorize", web::post().to(oauth::login))
            .route("/token", web::post().to(oauth::token)))
        .service(web::scope("/api")
            .service(web::scope("/v1")
                .service(web::scope("/users")
                    .route("/me", web::get().to(user::me))
                    .route("/register", web::post().to(user::register)))))
    ;
}


