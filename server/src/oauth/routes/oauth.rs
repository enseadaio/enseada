use std::str::FromStr;

use actix_session::Session as HttpSession;
use actix_web::body::Body;
use actix_web::error::{Error, InternalError, QueryPayloadError, UrlencodedError};
use actix_web::http::header;
use actix_web::web::{Data, Form, Json, Query};
use actix_web::web::{FormConfig, QueryConfig};
use actix_web::{get, post, ResponseError};
use actix_web::{HttpRequest, HttpResponse};
use actix_web_httpauth::extractors::basic::BasicAuth as BasicHeader;
use actix_web_httpauth::headers::authorization::{Basic, ParseError, Scheme};
use serde::{Deserialize, Serialize};
use url::Url;

use enseada::couchdb::repository::{Entity, Repository};
use enseada::secure;
use oauth::error::Error as OAuthError;
use oauth::error::ErrorKind;
use oauth::handler::{BasicAuth, RequestHandler};
use oauth::persistence::CouchStorage;
use oauth::request::{AuthorizationRequest, IntrospectionRequest, RevocationRequest, TokenRequest};
use oauth::response::{IntrospectionResponse, RevocationResponse, TokenResponse};
use oauth::session::Session;
use oauth::CouchOAuthHandler;
use users::UserService;

use crate::assets;
use crate::http::error::ApiError;
use crate::http::responses;
use crate::oauth::template::{LoginForm, Logout};
use crate::oauth::ErrorResponse;

type OAuthResult<T> = Result<T, ErrorResponse>;

#[derive(Debug, Deserialize)]
pub struct LoginPageQuery {
    pub error: Option<String>,
    #[serde(flatten)]
    pub auth_request: AuthorizationRequest,
}

#[get("/authorize")]
pub async fn login_form(
    handler: Data<CouchOAuthHandler>,
    users: Data<UserService>,
    query: Query<LoginPageQuery>,
    http_session: HttpSession,
    req: HttpRequest,
) -> OAuthResult<HttpResponse> {
    let client_auth = get_basic_auth(&req);
    let client_auth = client_auth.as_ref();
    let auth = &query.auth_request;
    if let Err(mut err) = handler.validate(auth, client_auth).await {
        log::error!("{}", err);
        err.set_state(auth.state.as_deref());
        return Err(err.into());
    }

    log::debug!(
        "Reading user session from cookie {:?}",
        http_session.get::<String>("user_id")?
    );

    let form = LoginForm {
        stylesheet_path: assets::stylesheet_path(),
        favicon_path: assets::icon_path(),
        logo_path: assets::logo_path(),
        response_type: auth.response_type.to_string(),
        client_id: auth.client_id.clone(),
        redirect_uri: auth.redirect_uri.clone(),
        scope: auth.scope.to_string(),
        state: auth.state.as_deref().unwrap_or_else(|| "").to_string(),
        error: query.error.clone(),
    };

    if let Some(username) = http_session.get::<String>("user_id")? {
        if let Some(_user) = users.find(&username).await? {
            return match do_login(
                handler,
                users,
                Form(LoginFormBody {
                    auth_request: auth.clone(),
                    username: String::from(""),
                    password: String::from(""),
                }),
                http_session,
                &req,
            )
            .await
            {
                Ok(res) => Ok(res),
                Err(err) => {
                    if let ErrorKind::AuthenticationFailed = err.kind() {
                        let mut form = form;
                        form.error = Some(err.to_string());
                        Ok(HttpResponse::Unauthorized()
                            .content_type("text/html; charset=utf-8")
                            .body(form.to_string()))
                    } else {
                        Err(err)
                    }
                }
            };
        }

        log::warn!(
            "User {} from session cookie cannot be found in database",
            username
        );
        http_session.remove("user_id");
    }

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
    handler: Data<CouchOAuthHandler>,
    users: Data<UserService>,
    form: Form<LoginFormBody>,
    http_session: HttpSession,
    req: HttpRequest,
) -> OAuthResult<HttpResponse> {
    match do_login(handler, users, form, http_session, &req).await {
        Ok(res) => Ok(res),
        Err(err) => {
            if let ErrorKind::AuthenticationFailed = err.kind() {
                if let Some(referer) = req.headers().get(http::header::REFERER) {
                    let mut ref_url = Url::parse(referer.to_str().unwrap())?;
                    ref_url
                        .query_pairs_mut()
                        .append_pair("error", &err.description());
                    Ok(HttpResponse::SeeOther()
                        .header(http::header::LOCATION, ref_url.to_string())
                        .finish())
                } else {
                    Err(err)
                }
            } else {
                Err(err)
            }
        }
    }
}

async fn do_login(
    handler: Data<CouchOAuthHandler>,
    users: Data<UserService>,
    form: Form<LoginFormBody>,
    http_session: HttpSession,
    req: &HttpRequest,
) -> OAuthResult<HttpResponse> {
    let client_auth = get_basic_auth(req);
    let client_auth = client_auth.as_ref();
    let form = form.into_inner();
    let auth = form.auth_request;
    let redirect_uri = auth.redirect_uri.clone();
    let mut url = Url::parse(&redirect_uri)?;

    let validate = handler.validate(&auth, client_auth).await;
    let client = match validate {
        Ok(client) => client,
        Err(mut err) => {
            err.set_state(auth.state.as_deref());
            return Ok(redirect_to_client(&mut url, err));
        }
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
            log::debug!("authentication failed");
            let err = OAuthError::with_state(
                ErrorKind::AuthenticationFailed,
                "authentication failed",
                auth.state.as_deref(),
            );
            return Err(ErrorResponse::from(err));
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
        Err(mut err) => {
            err.set_state(auth.state.as_deref());
            match err.kind() {
                ErrorKind::InvalidRedirectUri => Err(ErrorResponse::from(err)),
                _ => Ok(redirect_to_client(&mut url, err)),
            }
        }
    }
}

#[post("/token")]
pub async fn token(
    handler: Data<CouchOAuthHandler>,
    form: Form<TokenRequest>,
    req: HttpRequest,
) -> OAuthResult<Json<TokenResponse>> {
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
    handler: Data<CouchOAuthHandler>,
    form: Form<IntrospectionRequest>,
    req: HttpRequest,
) -> OAuthResult<Json<IntrospectionResponse>> {
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
    handler: Data<CouchOAuthHandler>,
    storage: Data<CouchStorage>,
    form: Form<RevocationRequest>,
    req: HttpRequest,
) -> OAuthResult<Json<RevocationResponse>> {
    let client_auth = get_basic_auth(&req);
    let client_auth = client_auth.as_ref();
    let req = form.into_inner();
    log::debug!("received revocation request");

    let client = handler.validate(&req, client_auth).await?;
    let session = &mut Session::for_client(client.client_id().to_string());
    let res = handler.handle(&req, session).await?;
    let sig = &secure::generate_signature(&req.token, handler.secret_key()).to_string();
    let pat = storage.find(sig).await?;
    if let Some(mut pat) = pat {
        pat.revoke();
        storage.save(pat).await?;
    }
    Ok(Json(res))
}

#[get("/logout")]
pub async fn logout(http_session: HttpSession) -> Result<Logout, ApiError> {
    if http_session.get::<String>("user_id")?.is_none() {
        return Err(ApiError::not_found("no active session found"));
    }

    http_session.clear();
    Ok(Logout::default())
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
                ErrorResponse(OAuthError::new(
                    ErrorKind::InvalidRedirectUri,
                    "invalid redirect_uri parameter",
                ))
                .error_response()
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
                    None => ErrorResponse(OAuthError::new(ErrorKind::InvalidRedirectUri, err))
                        .error_response(),
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
        UrlencodedError::Parse => OAuthError::new(
            ErrorKind::InvalidRequest,
            "request data is invalid or is missing a required parameter".to_string(),
        ),
        _ => OAuthError::new(ErrorKind::ServerError, &err),
    };
    InternalError::from_response(err, ErrorResponse(res).error_response()).into()
}
