use std::sync::Arc;

use actix_web::{HttpResponse, Responder, ResponseError};
use actix_web::web::{Data, Form, Json, Query, ServiceConfig};
use futures::TryFutureExt;
use url::Url;

use crate::error::ApiError;
use crate::oauth::RequestHandler;
use crate::oauth::scope::Scope;
use crate::oauth::error::{ErrorKind, Error as OAuthError};
use crate::oauth::handler::OAuthHandler;
use crate::oauth::request::{AuthorizationRequest, TokenRequest};
use crate::oauth::response::TokenResponse;
use crate::oauth::response::TokenType::Bearer;
use crate::oauth::persistence::CouchStorage;
use crate::responses;
use crate::templates::oauth::LoginForm;
use crate::couchdb::{self, db, Couch};
use std::collections::HashMap;


pub mod error;

type ConcreteOAuthHandler = OAuthHandler<CouchStorage, CouchStorage, CouchStorage, CouchStorage>;

pub fn add_oauth_handler(app: &mut ServiceConfig) {
    let couch = &couchdb::SINGLETON;
    let db = couch.database(db::name::OAUTH, true);
    let storage = Arc::new(CouchStorage::new(Arc::new(db)));
    let handler = OAuthHandler::new(
        storage.clone(),
        storage.clone(),
        storage.clone(),
        storage.clone()
    );
    app.data::<ConcreteOAuthHandler>(handler);
}

pub async fn login_form(handler: Data<ConcreteOAuthHandler>, query: Query<AuthorizationRequest>) -> impl Responder {
    let auth = query.into_inner();
    if let Err(err) = handler.validate(&auth).await {
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

pub async fn login(handler: Data<ConcreteOAuthHandler>, form: Form<AuthorizationRequest>) -> HttpResponse {
    let auth = form.into_inner();
    let validate = handler.validate(&auth);
    let handle = validate.and_then(|_| handler.handle(&auth)).await;
    let redirect_uri = auth.redirect_uri.clone();
    let mut url = Url::parse(&redirect_uri).unwrap();
    match handle {
        Ok(res) => redirect_back(&mut url, res),
        Err(err) => match err.kind() {
            ErrorKind::InvalidRedirectUri => HttpResponse::BadRequest().body(err.to_string()),
            _ => redirect_back(&mut url, err),
        }
    }
}

pub async fn token(handler: Data<ConcreteOAuthHandler>, form: Form<TokenRequest>) -> Result<Json<TokenResponse>, OAuthError> {
    let req = form.into_inner();
    log::debug!("received token request {:?}", &req);

    let validate = handler.validate(&req);
    let res: TokenResponse = validate.and_then(|_| handler.handle(&req)).await?;
    Ok(Json(res))

    // Uncomment to debug
    //     log::info!("received token request {:?}", &form);
    //     Ok(Json(TokenResponse {
    //         access_token: "access_token".to_string(),
    //         token_type: Bearer,
    //         expires_in: 3600,
    //         refresh_token: None,
    //         scope: Scope::from(vec!["profile", "email"]),
    //     }))
    }

pub fn redirect_back<T: serde::ser::Serialize>(redirect_uri: &mut Url, data: T) -> HttpResponse {
    let option = serde_urlencoded::to_string(data).ok();
    let query = option.as_deref();
    redirect_uri.set_query(query);
    log::debug!("redirecting to {}", &redirect_uri);
    responses::redirect_to(redirect_uri.to_string())
}