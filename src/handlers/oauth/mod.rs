use std::sync::Arc;

use actix_web::{HttpResponse, Responder};
use actix_web::web::{Data, Form, HttpRequest, Json, Query, ServiceConfig};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::errors::ApiError;
use crate::oauth::{RequestHandler, Scope};
use crate::oauth::error::ErrorKind;
use crate::oauth::handler::OAuthHandler;
use crate::oauth::request::{AuthorizationRequest, TokenRequest};
use crate::oauth::response::TokenResponse;
use crate::oauth::response::TokenType::Bearer;
use crate::oauth_impl::storage::CouchStorage;
use crate::responses;
use crate::templates::oauth::LoginForm;
use crate::oauth::storage::TokenStorage;
use crate::oauth_impl::token::AccessToken;

pub mod error;

pub fn add_oauth_handler(app: &mut ServiceConfig) {
    let storage = CouchStorage::new();
    let builder = &mut OAuthHandler::builder();
    builder.client_storage(&storage);
    builder.access_token_storage(&storage);
    builder.refresh_token_storage(&storage);
    builder.authorization_code_storage(&storage);
    let handler = builder.build().expect("failed to build OAuthHandler");
    app.data(handler);
}

pub async fn login_form(handler: Data<OAuthHandler>, auth: Query<AuthorizationRequest>) -> impl Responder {
    if let Err(err) = handler.validate(&auth) {
        log::error!("{}", err);
    }

    let response_type = auth.response_type.to_string();
    let client_id = auth.client_id.clone();
    let redirect_uri = auth.redirect_uri.clone();
    let scope = auth.scope.to_string();
    let state = auth.state.as_ref().unwrap_or(&"".to_string()).clone();

    LoginForm {
        response_type,
        client_id,
        redirect_uri,
        scope,
        state,
    }
}

pub async fn login(handler: Data<OAuthHandler>, auth: Form<AuthorizationRequest>) -> HttpResponse {
    let validate = handler.validate(&auth);
    let handle = validate.and_then(|_| handler.handle(&auth));
    let redirect_uri = auth.redirect_uri.clone();
    let mut url = Url::parse(&redirect_uri).unwrap();
    match handle {
        Ok(res) => redirect_back(&mut url, res),
        Err(err) => redirect_back(&mut url, err),
    }
}

pub async fn token(req: Form<TokenRequest>) -> Result<Json<TokenResponse>, ApiError> {
    log::info!("received token request {:?}", &req);
    Ok(Json(TokenResponse {
        access_token: "access_token".to_string(),
        token_type: Bearer,
        expires_in: 3600,
        refresh_token: None,
        scope: Scope::from(vec!["profile", "email"]),
    }))
}

pub fn redirect_back<T: serde::ser::Serialize>(redirect_uri: &mut Url, data: T) -> HttpResponse {
    let option = serde_urlencoded::to_string(data).ok();
    let query = option.as_deref();
    redirect_uri.set_query(query);
    log::debug!("redirecting to {}", &redirect_uri);
    responses::redirect_to(redirect_uri.to_string())
}