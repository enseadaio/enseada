use std::pin::Pin;

use actix_web::dev::{Payload, PayloadStream};
use actix_web::{FromRequest, HttpRequest};
use futures::Future;

use crate::http::error::ApiError;
use crate::http::extractor::session::TokenSession;
use oauth::scope::Scope;
use oauth::session::Session;
use std::ops::Deref;

pub struct OAuthScope(pub Scope);

impl FromRequest for OAuthScope {
    type Error = ApiError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut Payload<PayloadStream>) -> Self::Future {
        log::debug!("Extracting token scope from request");
        let session_fut = TokenSession::from_request(req, payload);
        Box::pin(async move {
            let TokenSession(session): TokenSession = session_fut.await?;
            log::debug!("Extracted session: {:?}", &session);
            Ok(OAuthScope(session.scope().clone()))
        })
    }
}

impl Deref for OAuthScope {
    type Target = Scope;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
