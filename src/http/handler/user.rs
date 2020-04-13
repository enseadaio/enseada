use actix_web::web::{Data, Form, Json, ServiceConfig};
use serde::{Deserialize, Serialize};

use crate::couchdb;
use crate::couchdb::db;
use crate::http::error::ApiError;
use crate::http::extractor::{scope::Scope, user::CurrentUser};
use crate::http::handler::ApiResult;
use crate::responses;
use crate::user::{User, UserService};

pub fn add_user_service(app: &mut ServiceConfig) {
    let couch = &couchdb::SINGLETON;
    let db = couch.database(db::name::USERS, true);
    let service = UserService::new(db);
    app.data(service);
}

#[derive(Debug, Serialize, PartialEq)]
pub struct MeResponse {
    pub username: String,
}

impl From<User> for MeResponse {
    fn from(user: User) -> Self {
        MeResponse { username: user.username().clone() }
    }
}

pub async fn me(
    user: CurrentUser,
    scope: Scope,
) -> ApiResult<Json<MeResponse>> {
    Scope::from("profile").matches(&scope)?;
    Ok(Json(user.into()))
}

#[derive(Debug, Serialize, PartialEq)]
pub struct UserResponse {
    pub username: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct RegistrationForm {
    pub username: String,
    pub password: String,
}

pub async fn register(
    service: Data<UserService>,
    data: Form<RegistrationForm>,
    scope: Scope,
) -> Result<Json<UserResponse>, ApiError> {
    Scope::from("users:manage").matches(&scope)?;
    let user = User::new(data.username.clone(), data.password.clone())?;
    let user = service.save_user(user).await?;
    responses::ok(UserResponse { username: user.username().clone() })
}
