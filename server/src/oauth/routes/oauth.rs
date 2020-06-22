use std::str::FromStr;

use actix_session::Session as HttpSession;
use actix_web::error::{Error, InternalError, QueryPayloadError, UrlencodedError};
use actix_web::http::header;
use actix_web::web::{Data, Form, Json, Query};
use actix_web::web::{FormConfig, QueryConfig};
use actix_web::{get, post};
use actix_web::{HttpRequest, HttpResponse};
use actix_web_httpauth::headers::authorization::{Basic, ParseError, Scheme};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::couchdb::repository::{Entity, Repository};
use crate::http::error::ApiError;
use crate::http::responses;
use crate::oauth::error::{Error as OAuthError, ErrorKind};
use crate::oauth::handler::{BasicAuth, RequestHandler};
use crate::oauth::request::{
    AuthorizationRequest, IntrospectionRequest, RevocationRequest, TokenRequest,
};
use crate::oauth::response::{IntrospectionResponse, RevocationResponse, TokenResponse};
use crate::oauth::session::Session;
use crate::oauth::template::LoginForm;
use crate::oauth::ConcreteOAuthHandler;
use crate::user::UserService;

#[get("/authorize")]
pub async fn login_form(
    handler: Data<ConcreteOAuthHandler>,
    users: Data<UserService>,
    query: Query<AuthorizationRequest>,
    http_session: HttpSession,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let client_auth = get_basic_auth(&req);
    let client_auth = client_auth.as_ref();
    let auth = query.into_inner();
    if let Err(err) = handler.validate(&auth, client_auth).await {
        log::error!("{}", err);
    }

    log::debug!(
        "Reading user session from cookie {:?}",
        http_session.get::<String>("user_id")?
    );

    if let Some(username) = http_session.get::<String>("user_id")? {
        if let Some(_user) = users.find(&username).await? {
            return do_login(
                handler,
                users,
                Form(LoginFormBody {
                    auth_request: auth,
                    username: String::from(""),
                    password: String::from(""),
                }),
                http_session,
                req,
            )
            .await;
        }

        log::warn!(
            "User {} from session cookie cannot be found in database",
            username
        );
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

#[post("/authorize")]
pub async fn login(
    handler: Data<ConcreteOAuthHandler>,
    users: Data<UserService>,
    form: Form<LoginFormBody>,
    http_session: HttpSession,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    do_login(handler, users, form, http_session, req).await
}

async fn do_login(
    handler: Data<ConcreteOAuthHandler>,
    users: Data<UserService>,
    form: Form<LoginFormBody>,
    http_session: HttpSession,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let client_auth = get_basic_auth(&req);
    let client_auth = client_auth.as_ref();
    let form = form.into_inner();
    let auth = form.auth_request;
    let redirect_uri = auth.redirect_uri.clone();
    let mut url = Url::parse(&redirect_uri)?;

    let validate = handler.validate(&auth, client_auth).await;
    let client = match validate {
        Ok(client) => client,
        Err(err) => return Ok(redirect_to_client(&mut url, err)),
    };

    let user = match http_session.get::<String>("user_id")? {
        Some(username) => users.find(&username).await?,
        None => users
            .authenticate_user(&form.username, &form.password)
            .await
            .ok(),
    };

    let user = match user {
        Some(user) => user,
        None => {
            log::debug!("Authentication failed");
            return Err(ApiError::Unauthorized(String::from(
                "authentication failed",
            )));
        }
    };

    log::debug!("Authentication successful");

    let user_id = user.id();
    http_session.set("user_id", user_id.id())?;
    let session = &mut Session::for_client(client.client_id().to_string());
    session.set_user_id(user_id.to_string());

    let handle = handler.handle(&auth, session).await;
    match handle {
        Ok(res) => Ok(redirect_to_client(&mut url, res)),
        Err(err) => match err.kind() {
            ErrorKind::InvalidRedirectUri => Err(ApiError::BadRequest(err.to_string())),
            _ => Ok(redirect_to_client(&mut url, err)),
        },
    }
}

#[post("/token")]
pub async fn token(
    handler: Data<ConcreteOAuthHandler>,
    form: Form<TokenRequest>,
    req: HttpRequest,
) -> Result<Json<TokenResponse>, OAuthError> {
    let client_auth = get_basic_auth(&req);
    let client_auth = client_auth.as_ref();
    let req = form.into_inner();
    log::debug!("received token request");

    let client = handler.validate(&req, client_auth).await?;
    let session = &mut Session::for_client(client.client_id().to_string());
    let res = handler.handle(&req, session).await?;
    Ok(Json(res))
}

#[post("/introspect")]
pub async fn introspect(
    handler: Data<ConcreteOAuthHandler>,
    form: Form<IntrospectionRequest>,
    req: HttpRequest,
) -> Result<Json<IntrospectionResponse>, OAuthError> {
    let client_auth = get_basic_auth(&req);
    let client_auth = client_auth.as_ref();
    let req = form.into_inner();
    log::debug!("received introspection request");

    let client = handler.validate(&req, client_auth).await?;
    let session = &mut Session::for_client(client.client_id().to_string());
    let res = handler.handle(&req, session).await?;
    Ok(Json(res))
}

#[post("/revoke")]
pub async fn revoke(
    handler: Data<ConcreteOAuthHandler>,
    form: Form<RevocationRequest>,
    req: HttpRequest,
) -> Result<Json<RevocationResponse>, OAuthError> {
    let client_auth = get_basic_auth(&req);
    let client_auth = client_auth.as_ref();
    let req = form.into_inner();
    log::debug!("received revocation request");

    let client = handler.validate(&req, client_auth).await?;
    let session = &mut Session::for_client(client.client_id().to_string());
    let res = handler.handle(&req, session).await?;
    Ok(Json(res))
}

pub fn redirect_to_client<T: Serialize>(redirect_uri: &mut Url, data: T) -> HttpResponse {
    let option = serde_urlencoded::to_string(data).ok();
    let query = option.as_deref();
    redirect_uri.set_query(query);
    log::debug!("redirecting to {}", &redirect_uri);
    responses::redirect_to(redirect_uri.to_string())
}

fn get_basic_auth(req: &HttpRequest) -> Option<BasicAuth> {
    req.headers()
        .get(header::AUTHORIZATION)
        .map(Basic::parse)
        .and_then(Result::<Basic, ParseError>::ok)
        .map(|basic| {
            BasicAuth::new(
                basic.user_id().to_string(),
                basic.password().map(ToString::to_string),
            )
        })
}

pub fn handle_query_errors(cfg: QueryConfig) -> QueryConfig {
    cfg.error_handler(handle_query_error)
}

pub fn handle_form_errors(cfg: FormConfig) -> FormConfig {
    cfg.error_handler(handle_form_error)
}

fn handle_query_error(err: QueryPayloadError, req: &HttpRequest) -> Error {
    let detail = err.to_string();
    log::error!("Error: {}", &detail);
    let res = match &err {
        QueryPayloadError::Deserialize(err) => {
            if detail.contains("redirect_uri") {
                HttpResponse::BadRequest().body("invalid redirect_uri parameter")
            } else {
                let err = OAuthError::new(ErrorKind::InvalidRequest, err.to_string());
                let result =
                    serde_urlencoded::from_str::<Vec<(String, String)>>(req.query_string());
                match result
                    .ok()
                    .unwrap_or_else(Vec::new)
                    .into_iter()
                    .find(|(k, _)| k == "redirect_uri")
                    .and_then(|(_, uri)| Url::from_str(uri.as_str()).ok())
                {
                    Some(mut redirect_uri) => redirect_to_client(&mut redirect_uri, err),
                    None => HttpResponse::BadRequest().body("invalid redirect_uri parameter"),
                }
            }
        }
    };
    InternalError::from_response(err, res).into()
}

fn handle_form_error(err: UrlencodedError, req: &HttpRequest) -> Error {
    let detail = err.to_string();
    log::error!("Error: {}", &detail);
    log::debug!("{:?}", req);
    let res = match &err {
        UrlencodedError::Parse => HttpResponse::BadRequest().json(OAuthError::new(
            ErrorKind::InvalidRequest,
            "request data is invalid or is missing a required parameter".to_string(),
        )),
        _ => HttpResponse::BadRequest()
            .content_type("text/plain")
            .body(detail),
    };
    InternalError::from_response(err, res).into()
}
