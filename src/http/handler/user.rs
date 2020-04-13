use actix_web::web::{Data, Form, Json, ServiceConfig};
use serde::{Deserialize, Serialize};

use crate::couchdb;
use crate::couchdb::db;
use crate::error::ApiError;
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

pub async fn me(service: Data<UserService>) -> ApiResult<Json<MeResponse>> {
    log::info!("Getting myself");
    Ok(Json(MeResponse { username: String::from("random") }))
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

pub async fn register(service: Data<UserService>, data: Form<RegistrationForm>) -> Result<Json<UserResponse>, ApiError> {
    let user = User::new(data.username.clone(), data.password.clone())?;
    let user = service.save_user(user).await?;
    responses::ok(UserResponse { username: user.username().clone() })
}
