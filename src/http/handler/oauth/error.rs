use std::str::FromStr;

use actix_web::{HttpRequest, HttpResponse};
use actix_web::error::{Error, InternalError, QueryPayloadError, UrlencodedError};
use actix_web::web::{FormConfig, QueryConfig};
use url::Url;

use crate::http::handler::oauth::redirect_to_client;
use crate::oauth::error::{Error as OAuthError, ErrorKind};

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
                let result = serde_urlencoded::from_str::<Vec<(String, String)>>(req.query_string());
                match result.ok()
                    .unwrap_or_else(Vec::new)
                    .into_iter().find(|(k, _)| { k == "redirect_uri" })
                    .and_then(|(_, uri)| Url::from_str(uri.as_str()).ok()){
                    Some(mut redirect_uri) => redirect_to_client(&mut redirect_uri, err),
                    None => HttpResponse::BadRequest().body("invalid redirect_uri parameter")
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
         UrlencodedError::Parse => HttpResponse::BadRequest().json(OAuthError::new(ErrorKind::InvalidRequest, "request data is invalid or is missing a required parameter".to_string())),
        _ => HttpResponse::BadRequest().content_type("text/plain").body(detail),
    };
    InternalError::from_response(err, res).into()
}