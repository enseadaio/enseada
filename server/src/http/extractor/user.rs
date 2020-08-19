use std::ops::Deref;
use std::pin::Pin;

use actix_web::dev::{Payload, PayloadStream};
use actix_web::web::Data;
use actix_web::{FromRequest, HttpRequest};
use futures::Future;

use enseada::couchdb::repository::{Entity, Repository};
use enseada::guid::Guid;
use users::{User, UserService};

use crate::http::error::ApiError;
use crate::http::extractor::session::TokenSession;

pub struct CurrentUser(User);

impl FromRequest for CurrentUser {
    type Error = ApiError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut Payload<PayloadStream>) -> Self::Future {
        log::debug!("extracting current user from request");
        let service_fut = Data::<UserService>::from_request(req, payload);
        let session_fut = TokenSession::from_request(req, payload);
        Box::pin(async move {
            let service = service_fut.await?;
            let session: TokenSession = session_fut.await?;
            let username = match session.user_id() {
                Some(username) => username,
                None => return Err(ApiError::Unauthorized("unauthorized".to_string())),
            };

            let guid = Guid::from(username.clone());
            let user = service.find(guid.id()).await?;
            match user {
                Some(user) => {
                    log::debug!("Found user {}", user.id());
                    Ok(CurrentUser(user))
                }
                None => Err(ApiError::Unauthorized("unauthorized".to_string())),
            }
        })
    }
}

impl Deref for CurrentUser {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
