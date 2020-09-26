use std::sync::Arc;

use actix_web::get;
use actix_web::web::{self, Json, ServiceConfig};
use actix_web::FromRequest;
use serde::Serialize;
use url::Url;

use ::oauth::handler::OAuthHandler;
use ::oauth::persistence::CouchStorage;
use ::oauth::request::{AuthorizationRequest, TokenRequest};
use enseada::couchdb::db::Database;

use crate::config::CONFIG;

mod api;
mod oauth;

pub fn mount(db: Database) -> Box<impl FnOnce(&mut ServiceConfig)> {
    Box::new(|cfg: &mut ServiceConfig| {
        let storage = Arc::new(CouchStorage::new(db.clone()));
        let handler = OAuthHandler::new(
            storage.clone(),
            storage.clone(),
            storage.clone(),
            storage,
            CONFIG.secret_key(),
        );

        cfg.data(CouchStorage::new(db));
        cfg.data(handler);

        cfg.service(oauth::metadata);
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
                .service(oauth::revoke)
                .service(oauth::logout),
        );

        // OAuth Clients
        cfg.service(api::client::list);
        cfg.service(api::client::create);
        cfg.service(api::client::get);
        cfg.service(api::client::update);
        cfg.service(api::client::delete);

        // Personal Access Tokens
        cfg.service(api::pat::list);
        cfg.service(api::pat::create);
        cfg.service(api::pat::get);
        cfg.service(api::pat::delete);
    })
}
