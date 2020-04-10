use std::sync::Arc;

use actix_web::{HttpResponse, Responder, HttpRequest, Error};
use actix_web::web::{Data, Form, Json, Query, ServiceConfig};
use actix_session::Session as HttpSession;
use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::couchdb::{self, db};
use crate::error::ApiError;
use crate::oauth::error::{Error as OAuthError, ErrorKind};
use crate::oauth::handler::OAuthHandler;
use crate::oauth::persistence::CouchStorage;
use crate::oauth::request::{AuthorizationRequest, TokenRequest};
use crate::oauth::RequestHandler;
use crate::oauth::response::TokenResponse;
use crate::oauth::session::Session;
use crate::responses;
use crate::templates::oauth::LoginForm;
use crate::user::UserService;
use actix_web::body::Body;

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
        storage.clone(),
    );
    app.data::<ConcreteOAuthHandler>(handler);
}

pub async fn login_form(
    handler: Data<ConcreteOAuthHandler>,
    users: Data<UserService>,
    query: Query<AuthorizationRequest>,
    http_session: HttpSession,
) -> Result<HttpResponse, ApiError> {
    let auth = query.into_inner();
    if let Err(err) = handler.validate(&auth).await {
        log::error!("{}", err);
    }

    log::debug!("Reading user session from cookie {:?}", http_session.get::<String>("user_id")?);

    if let Some(username) = http_session.get::<String>("user_id")? {
        if let Some(user) = users.find_user(&username).await? {
            return login(handler, users, Form(LoginFormBody {
                auth_request: auth,
                username: String::from(""),
                password: String::from(""),
            }), http_session).await
        }

        log::warn!("User {} from session cookie cannot be found in database", username);
        http_session.remove("user_id");
    }

    let form = LoginForm {
        response_type: auth.response_type.to_string(),
        client_id: auth.client_id.clone(),
        redirect_uri: auth.redirect_uri.clone(),
        scope: auth.scope.to_string(),
        state: auth.state.as_ref().unwrap_or(&"".to_string()).clone(),
    };

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(form.to_string()))
}

#[derive(Debug, Deserialize)]
pub struct LoginFormBody {
    pub username: String,
    pub password: String,
    #[serde(flatten)]
    pub auth_request: AuthorizationRequest,
}

pub async fn login(
    handler: Data<ConcreteOAuthHandler>,
    users: Data<UserService>,
    form: Form<LoginFormBody>,
    http_session: HttpSession,
) -> Result<HttpResponse, ApiError> {
    let form = form.into_inner();
    let auth = form.auth_request;
    let redirect_uri = auth.redirect_uri.clone();
    let mut url = Url::parse(&redirect_uri).unwrap();

    let validate = handler.validate(&auth).await;
    if let Err(err) = validate {
        return Ok(redirect_to_client(&mut url, err));
    }

    let user = match http_session.get::<String>("user_id")? {
        Some(username) => users.find_user(&username).await?,
        None => users.authenticate_user(form.username, form.password).await.ok(),
    };

    let user = match user {
        Some(user) => user,
        None => {
            log::debug!("Authentication failed");
            return Err(ApiError::Unauthorized(String::from("authentication failed")));
        }
    };

    log::debug!("Authentication successful");

    let user_id = user.id();
    http_session.set("user_id", user_id.id())?;
    let session = &mut Session::empty();
    session.set_user_id(user_id.to_string());

    let handle = handler.handle(&auth, session).await;
    match handle {
        Ok(res) => Ok(redirect_to_client(&mut url, res)),
        Err(err) => match err.kind() {
            ErrorKind::InvalidRedirectUri => Err(ApiError::BadRequest(err.to_string())),
            _ => Ok(redirect_to_client(&mut url, err)),
        }
    }
}

pub async fn token(handler: Data<ConcreteOAuthHandler>, form: Form<TokenRequest>) -> Result<Json<TokenResponse>, OAuthError> {
    let req = form.into_inner();
    log::debug!("received token request {:?}", &req);

    let session = &mut Session::empty();
    let validate = handler.validate(&req);
    let res: TokenResponse = validate.and_then(|_| handler.handle(&req, session)).await?;
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

pub fn redirect_to_client<T: Serialize>(redirect_uri: &mut Url, data: T) -> HttpResponse {
    let option = serde_urlencoded::to_string(data).ok();
    let query = option.as_deref();
    redirect_uri.set_query(query);
    log::debug!("redirecting to {}", &redirect_uri);
    responses::redirect_to(redirect_uri.to_string())
}