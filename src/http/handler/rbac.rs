use std::sync::RwLock;

use actix_web::web::{Data, Json, Path};
use serde::{Deserialize, Serialize};

use crate::http::error::ApiError;
use crate::http::extractor::user::CurrentUser;
use crate::http::handler::ApiResult;
use crate::http::handler::user::UsernamePathParam;
use crate::oauth::scope::Scope;
use crate::rbac::Enforcer;
use crate::user::UserService;

#[derive(Debug, Serialize, PartialEq)]
pub struct RoleResponse {
    pub username: String,
    pub roles: Vec<String>,
}

pub async fn get_user_roles(
    service: Data<UserService>,
    enforcer: Data<RwLock<Enforcer>>,
    scope: Scope,
    current_user: CurrentUser,
    path: Path<UsernamePathParam>,
) -> ApiResult<Json<RoleResponse>> {
    Scope::from(vec!["users:read", "roles"]).matches_exactly(&scope)?;
    let enforcer = enforcer.read().unwrap();
    enforcer.check(current_user.id().id(), "roles", "read")?;

    let username = &path.username;
    if service.find_user(username).await?.is_none() {
        return Err(ApiError::NotFound(format!("User {} not found", username)));
    }

    let roles = enforcer.get_principal_roles(username).await?;
    Ok(Json(RoleResponse {
        username: username.clone(),
        roles,
    }))
}

#[derive(Debug, Deserialize)]
pub struct UserRolesPathParams {
    username: String,
    role: String,
}

pub async fn add_user_role(
    service: Data<UserService>,
    enforcer: Data<RwLock<Enforcer>>,
    scope: Scope,
    current_user: CurrentUser,
    path: Path<UserRolesPathParams>,
) -> ApiResult<Json<RoleResponse>> {
    Scope::from(vec!["users:manage", "roles"]).matches_exactly(&scope)?;
    let enforcer = enforcer.read().unwrap();
    enforcer.check(current_user.id().id(), "roles", "manage")?;

    let username = &path.username;
    if service.find_user(username).await?.is_none() {
        return Err(ApiError::NotFound(format!("User {} not found", username)));
    }

    enforcer.add_role_to_principal(username, &path.role).await?;
    let roles = enforcer.get_principal_roles(username).await?;
    Ok(Json(RoleResponse {
        username: username.clone(),
        roles,
    }))
}

pub async fn remove_user_role(
    service: Data<UserService>,
    enforcer: Data<RwLock<Enforcer>>,
    scope: Scope,
    current_user: CurrentUser,
    path: Path<UserRolesPathParams>,
) -> ApiResult<Json<RoleResponse>> {
    Scope::from(vec!["users:manage", "roles"]).matches_exactly(&scope)?;
    let enforcer = enforcer.read().unwrap();
    enforcer.check(current_user.id().id(), "roles", "manage")?;

    let username = &path.username;
    if service.find_user(username).await?.is_none() {
        return Err(ApiError::NotFound(format!("User {} not found", username)));
    }

    enforcer.remove_role_from_principal(username, &path.role).await?;
    let roles = enforcer.get_principal_roles(username).await?;
    Ok(Json(RoleResponse {
        username: username.clone(),
        roles,
    }))
}