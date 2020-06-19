use std::sync::Arc;

use actix_web::web;
use actix_web::web::ServiceConfig;
use actix_web::FromRequest;

use crate::oauth::handler::OAuthHandler;
use crate::oauth::persistence::CouchStorage;
use crate::oauth::request::{AuthorizationRequest, TokenRequest};

mod api;
mod oauth;

pub fn mount(cfg: &mut ServiceConfig) {
    let couch = &crate::couchdb::SINGLETON;
    let db = Arc::new(couch.database(crate::couchdb::name::OAUTH, true));
    let storage = Arc::new(CouchStorage::new(db.clone()));
    let handler = OAuthHandler::new(storage.clone(), storage.clone(), storage.clone(), storage);

    cfg.data(CouchStorage::new(db.clone()));
    cfg.data(handler);

    cfg.service(
        web::scope("/oauth")
            .app_data(web::Query::<AuthorizationRequest>::configure(
                oauth::handle_query_errors,
            ))
            .app_data(web::Form::<TokenRequest>::configure(
                oauth::handle_form_errors,
            ))
            .service(oauth::login_form)
            .service(oauth::login)
            .service(oauth::token)
            .service(oauth::introspect)
            .service(oauth::revoke),
    );

    cfg.service(api::list_clients);
    cfg.service(api::create_client);
    cfg.service(api::get_client);
    cfg.service(api::update_client);
    cfg.service(api::delete_client);
}
