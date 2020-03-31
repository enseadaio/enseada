use actix_web::dev::ServiceResponse;
use actix_web::{Result, HttpRequest, HttpResponse, FromRequest};
use actix_web::middleware::errhandlers::ErrorHandlerResponse;
use actix_web::error::{InternalError, Error, UrlencodedError, QueryPayloadError};
use actix_web::web::{JsonConfig, FormConfig, QueryConfig, Query};
use url::Url;
use crate::oauth::response::TokenResponse;
use crate::handlers::oauth::redirect_back;
use crate::oauth::error::{Error as OAuthError, ErrorKind};
use std::str::FromStr;

pub fn handle_query_errors(cfg: QueryConfig) -> QueryConfig {
    cfg.error_handler(handle_query_error)
}

pub fn handle_form_errors(cfg: FormConfig) -> FormConfig {
    cfg.error_handler(handle_form_error)
}

fn handle_query_error(err: QueryPayloadError, req: &HttpRequest) -> Error {
    let detail = err.to_string();
    let res = match &err {
        QueryPayloadError::Deserialize(err) => {
            log::error!("{:?}", err);
            if detail.contains("redirect_uri") {
                HttpResponse::BadRequest().body("invalid redirect_uri parameter")
            } else {
                let err = OAuthError::new(ErrorKind::InvalidRequest, err.to_string());
                let result = serde_urlencoded::from_str::<Vec<(String, String)>>(req.query_string());
                match result.ok()
                    .unwrap_or_else(Vec::new)
                    .into_iter().find(|(k, _)| { k == "redirect_uri" })
                    .and_then(|(_, uri)| Url::from_str(uri.as_str()).ok()){
                    Some(mut redirect_uri) => redirect_back(&mut redirect_uri, err),
                    None => HttpResponse::BadRequest().body("invalid redirect_uri parameter")
                }
            }
        },
        _ => HttpResponse::BadRequest().content_type("text/plain").body(detail),
    };
    InternalError::from_response(err, res).into()
}

fn handle_form_error(err: UrlencodedError, _req: &HttpRequest) -> Error {
    let detail = err.to_string();
    let res = match &err {
         UrlencodedError::Parse => HttpResponse::BadRequest().json(OAuthError::new(ErrorKind::InvalidRequest, "request data is invalid or is missing a required parameter".to_string())),
        _ => HttpResponse::BadRequest().content_type("text/plain").body(detail),
    };
    InternalError::from_response(err, res).into()
}