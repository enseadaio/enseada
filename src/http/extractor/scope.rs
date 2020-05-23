use std::pin::Pin;

use actix_web::{FromRequest, HttpRequest};
use actix_web::dev::{Payload, PayloadStream};
use futures::Future;

use crate::http::error::ApiError;
use crate::http::extractor::session::TokenSession;
use crate::oauth::scope::Scope as OAuthScope;
use crate::oauth::session::Session;

pub type Scope = OAuthScope;

impl FromRequest for Scope {
    type Error = ApiError;
    type Future = Pin<Box<dyn Future<Output=Result<Self, Self::Error>>>>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut Payload<PayloadStream>) -> Self::Future {
        log::debug!("Extracting token scope from request");
        let session_fut = TokenSession::from_request(req, payload);
        Box::pin(async move {
            let session: Session = session_fut.await?;
            log::debug!("Extracted session: {:?}", &session);
            Ok(session.scope().clone())
        })
    }
}