use std::ops::Deref;
use std::pin::Pin;

use actix_web::dev::{Payload, PayloadStream};
use actix_web::http::header;
use actix_web::web::Data;
use actix_web::{FromRequest, HttpRequest};
use actix_web_httpauth::headers::authorization::{Basic, Bearer, ParseError, Scheme};
use futures::Future;

use oauth::error::ErrorKind;
use oauth::handler::TokenIntrospectionHandler;
use oauth::session::Session;
use oauth::token::{AccessToken, Token};
use oauth::{CouchOAuthHandler, Expirable};

use crate::http::error::ApiError;

#[derive(Debug)]
pub struct TokenSession(Session);

impl FromRequest for TokenSession {
    type Error = ApiError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut Payload<PayloadStream>) -> Self::Future {
        log::debug!("extracting token session from request");
        let handler_fut = Data::<CouchOAuthHandler>::from_request(req, payload);
        let header = req.headers().get(header::AUTHORIZATION);
        let token = header
            .map(Bearer::parse)
            .and_then(Result::<Bearer, ParseError>::ok)
            .map(|bearer| bearer.token().clone())
            .or_else(|| {
                header
                    .map(Basic::parse)
                    .and_then(Result::<Basic, ParseError>::ok)
                    .and_then(|basic| {
                        let username = basic.user_id();
                        if username.ne("x-oauth-token") {
                            None
                        } else {
                            basic.password().cloned()
                        }
                    })
            });
        Box::pin(async move {
            match token {
                Some(token) => {
                    log::debug!("token found in header, fetching from db");
                    let oauth_handler = handler_fut.await?;
                    let access_token: AccessToken = oauth_handler.get_token(&token).await?;
                    if access_token.is_expired() {
                        log::debug!("token is expired");
                        TokenIntrospectionHandler::<AccessToken>::revoke_token(
                            oauth_handler.get_ref(),
                            &token,
                        )
                        .await?;
                        Err(ApiError::unauthorized())
                    } else {
                        log::debug!("token is valid");
                        Ok(TokenSession(access_token.session().clone()))
                    }
                }
                None => {
                    log::debug!("no token found in header");
                    Err(ApiError::unauthorized())
                }
            }
        })
    }
}

impl Deref for TokenSession {
    type Target = Session;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
