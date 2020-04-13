use std::fmt::Debug;
use std::pin::Pin;

use actix_session::UserSession;
use actix_web::{Error, FromRequest, HttpRequest};
use actix_web::dev::{Payload, PayloadStream, ServiceRequest};
use actix_web::error::PayloadError;
use actix_web::http::{header, HeaderMap};
use actix_web::web::{Bytes, Data};
use actix_web_httpauth::extractors::basic::BasicAuth;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web_httpauth::headers::authorization::{Basic, Bearer, ParseError, Scheme};
use futures::{Future, FutureExt, Stream, TryFutureExt};
use serde::export::Formatter;

use crate::http::error::ApiError;
use crate::http::handler::oauth::ConcreteOAuthHandler;
use crate::oauth::Expirable;
use crate::oauth::handler::TokenIntrospectionHandler;
use crate::oauth::session::Session;
use crate::oauth::token::AccessToken;

pub type TokenSession = Session;

impl FromRequest for TokenSession {
    type Error = ApiError;
    type Future = Pin<Box<dyn Future<Output=Result<Self, Self::Error>>>>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut Payload<PayloadStream>) -> Self::Future {
        log::debug!("Extracting token session from request");
        let handler_fut = Data::<ConcreteOAuthHandler>::from_request(req, payload);
        let header = req.headers().get(header::AUTHORIZATION);
        let token = header.map(Bearer::parse)
            .and_then(Result::<Bearer, ParseError>::ok)
            .map(|bearer| bearer.token().clone())
            .or_else(|| header.map(Basic::parse)
                .and_then(Result::<Basic, ParseError>::ok)
                .and_then(|basic| {
                    let username = basic.user_id();
                    if username.ne("x-oauth-token") {
                        None
                    } else {
                        basic.password().cloned()
                    }
                }));
        Box::pin(async move {
            match token {
                Some(token) => {
                    log::debug!("Token found");
                    let oauth_handler = handler_fut.await?;
                    let access_token: AccessToken = oauth_handler.get_token(&token).await.map_err(|_| ApiError::unauthorized())?;
                    if access_token.is_expired() {
                        log::debug!("Token is expired");
                        TokenIntrospectionHandler::<AccessToken>::revoke_token(oauth_handler.get_ref(), &token).await.map_err(|_| ApiError::unauthorized())?;
                        Err(ApiError::unauthorized())
                    } else {
                        log::debug!("Token is valid");
                        Ok(access_token.session().clone())
                    }
                }
                None => {
                    log::debug!("Token not found");
                    Err(ApiError::unauthorized())
                }
            }
        })
    }
}
