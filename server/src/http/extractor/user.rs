use std::pin::Pin;

use actix_web::{Error, FromRequest, HttpRequest};
use actix_web::dev::{Payload, PayloadStream};
use actix_web::error::PayloadError;
use actix_web::web::{Bytes, Data};
use futures::{Future, FutureExt, Stream, TryFutureExt};

use crate::guid::Guid;
use crate::http::error::ApiError;
use crate::http::extractor::session::TokenSession;
use crate::user::{User, UserService};

pub type CurrentUser = User;

impl FromRequest for CurrentUser {
    type Error = ApiError;
    type Future = Pin<Box<dyn Future<Output=Result<Self, Self::Error>>>>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut Payload<PayloadStream>) -> Self::Future {
        log::debug!("Extracting current user from request");
        let service_fut = Data::<UserService>::from_request(req, payload);
        let session_fut = TokenSession::from_request(req, payload);
        Box::pin(async move {
            let service = service_fut.await?;
            let session: TokenSession = session_fut.await?;
            let username = match session.user_id() {
                Some(username) => username,
                None => return Err(ApiError::Unauthorized("unauthorized".to_string()))
            };

            let guid = Guid::from(username.clone());
            let user = service.find_user(guid.id()).await?;
            match user {
                Some(user) => {
                    log::debug!("Found user {}", user.id());
                    Ok(user)
                },
                None => Err(ApiError::Unauthorized("unauthorized".to_string())),
            }
        })
    }
}