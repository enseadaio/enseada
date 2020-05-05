use actix_files as fs;
use actix_web::{FromRequest, web};

use crate::http::handler::{api_docs, health, oauth, rbac, ui, user};
use crate::oauth::request::{AuthorizationRequest, TokenRequest};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(fs::Files::new("/static", "./dist").show_files_listing())
        .route("/health", web::get().to(health::get))

        // UI
        .service(web::scope("/ui")
            .route("", web::get().to(ui::index))
        )

        // OAuth
        .service(web::scope("/oauth")
            .app_data(web::Query::<AuthorizationRequest>::configure(oauth::error::handle_query_errors))
            .app_data(web::Form::<TokenRequest>::configure(oauth::error::handle_form_errors))
            .service(web::resource("/authorize")
                .route(web::get().to(oauth::login_form))
                .route(web::post().to(oauth::login)))
            .route("/token", web::post().to(oauth::token))
            .route("/introspect", web::post().to(oauth::introspect))
            .route("/revoke", web::post().to(oauth::revoke))
        )

        // API
        .service(web::scope("/api")
            .service(web::scope("/docs")
                .service(web::resource("/openapi.yml")
                    .name("open_api_spec")
                    .route(web::get().to(api_docs::open_api))
                )
                .route("", web::get().to(api_docs::redoc))
            )

            // V1Beta1
            .service(web::scope("/v1beta1")
                .service(web::scope("/users")
                    .service(web::resource("")
                        .route(web::get().to(user::list))
                        .route(web::post().to(user::register))
                    )
                    .route("/me", web::get().to(user::me))
                    .service(web::scope("/{username}")
                        .route("", web::get().to(user::get))
                        .route("", web::delete().to(user::delete))
                        .service(web::scope("/roles")
                            .route("", web::get().to(rbac::get_user_roles))
                            .route("/{role}", web::put().to(rbac::add_user_role))
                            .route("/{role}", web::delete().to(rbac::remove_user_role))
                        )
                        .service(web::resource("/permissions")
                            .route(web::get().to(rbac::get_user_permissions))
                            .route(web::post().to(rbac::add_user_permission))
                            .route(web::delete().to(rbac::remove_user_permission))
                        )
                    )
                )
                .service(web::scope("/roles")
                    .service(web::scope("/{role}")
                        .service(web::resource("/permissions")
                            .route(web::get().to(rbac::get_role_permissions))
                            .route(web::post().to(rbac::add_role_permission))
                            .route(web::delete().to(rbac::remove_role_permission))
                        )
                    )
                )
            )
        )
    ;
}


