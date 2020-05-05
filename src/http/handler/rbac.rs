use std::sync::RwLock;

use actix_web::web::{Data, Json, Path, Query};
use serde::{Deserialize, Serialize};

use crate::guid::Guid;
use crate::http::error::ApiError;
use crate::http::extractor::user::CurrentUser;
use crate::http::handler::{ApiResult, PaginationQuery};
use crate::http::handler::user::UsernamePathParam;
use crate::oauth::scope::Scope;
use crate::pagination::{Cursor, Page};
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
    let username = &path.username;
    let enforcer = enforcer.read().unwrap();
    let sub = Guid::partitioned("user", username);
    enforcer.check(current_user.id(), &sub, "read_roles")?;

    if service.find_user(username).await?.is_none() {
        return Err(ApiError::NotFound(format!("User {} not found", username)));
    }

    let roles = enforcer.get_principal_roles(&sub).await?;
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
    let username = &path.username;
    let sub = Guid::partitioned("user", username);
    enforcer.check(current_user.id(), &sub, "manage_roles")?;

    if service.find_user(username).await?.is_none() {
        return Err(ApiError::NotFound(format!("User {} not found", username)));
    }

    enforcer.add_role_to_principal(sub.clone(), &path.role).await?;
    let roles = enforcer.get_principal_roles(&sub).await?;
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
    let username = &path.username;
    let sub = &Guid::partitioned("user", username);
    enforcer.check(current_user.id(), sub, "manage_roles")?;

    if service.find_user(username).await?.is_none() {
        return Err(ApiError::NotFound(format!("User {} not found", username)));
    }

    enforcer.remove_role_from_principal(sub, &path.role).await?;
    let roles = enforcer.get_principal_roles(sub).await?;
    Ok(Json(RoleResponse {
        username: username.clone(),
        roles,
    }))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Permission {
    #[serde(skip_serializing_if = "Option::is_none")]
    subject: Option<Guid>,
    object: Guid,
    action: String,
}

pub async fn get_user_permissions(
    enforcer: Data<RwLock<Enforcer>>,
    scope: Scope,
    current_user: CurrentUser,
    path: Path<UsernamePathParam>,
    list: Query<PaginationQuery>,
) -> ApiResult<Json<Page<Permission>>> {
    Scope::from(vec!["users:read", "permissions"]).matches_exactly(&scope)?;
    let username = &path.username;
    let enforcer = enforcer.read().unwrap();
    let sub = &Guid::partitioned("user", username);
    enforcer.check(current_user.id(), sub, "read_permissions")?;

    let limit = list.limit();
    let cursor = list.cursor();

    let cursor = if let Some(cursor) = cursor {
        Some(Cursor::from_b64(cursor)?)
    } else {
        None
    };

    let rules = enforcer.list_principal_permissions(&sub, limit, cursor.as_ref()).await?;
    let permissions = rules.map(|rule| Permission {
        subject: Some(rule.subject().clone()),
        object: rule.object().clone(),
        action: rule.action().to_string(),
    });
    Ok(Json(permissions))
}

pub async fn add_user_permission(
    enforcer: Data<RwLock<Enforcer>>,
    scope: Scope,
    current_user: CurrentUser,
    path: Path<UsernamePathParam>,
    permission: Json<Permission>,
) -> ApiResult<Json<Permission>> {
    Scope::from(vec!["users:read", "permissions"]).matches_exactly(&scope)?;
    let username = &path.username;
    let enforcer = enforcer.read().unwrap();
    let sub = Guid::partitioned("user", username);
    enforcer.check(current_user.id(), &sub, "manage_permissions")?;

    let mut permission = permission;
    permission.subject = Some(sub.clone());
    enforcer.add_permission(sub, permission.object.clone(), &permission.action).await?;

    Ok(permission)
}

pub async fn remove_user_permission(
    enforcer: Data<RwLock<Enforcer>>,
    scope: Scope,
    current_user: CurrentUser,
    path: Path<UsernamePathParam>,
    permission: Json<Permission>,
) -> ApiResult<Json<Permission>> {
    Scope::from(vec!["users:read", "permissions"]).matches_exactly(&scope)?;
    let username = &path.username;
    let enforcer = enforcer.read().unwrap();
    let sub = &Guid::partitioned("user", username);
    enforcer.check(current_user.id(), sub, "manage_permissions")?;

    let mut permission = permission;
    permission.subject = Some(sub.clone());
    enforcer.remove_permission(sub, permission.object.clone(), &permission.action).await?;

    Ok(permission)
}

#[derive(Debug, Deserialize)]
pub struct RolePathParam {
    pub role: String,
}

pub async fn get_role_permissions(
    enforcer: Data<RwLock<Enforcer>>,
    scope: Scope,
    current_user: CurrentUser,
    path: Path<RolePathParam>,
    list: Query<PaginationQuery>,
) -> ApiResult<Json<Page<Permission>>> {
    Scope::from(vec!["roles", "permissions"]).matches_exactly(&scope)?;
    let role = &path.role;
    let enforcer = enforcer.read().unwrap();
    let sub = &Guid::partitioned("role", role);
    enforcer.check(current_user.id(), sub, "read_permissions")?;

    let limit = list.limit();
    let cursor = list.cursor();

    let cursor = if let Some(cursor) = cursor {
        Some(Cursor::from_b64(cursor)?)
    } else {
        None
    };

    let rules = enforcer.list_principal_permissions(&sub, limit, cursor.as_ref()).await?;
    let permissions = rules.map(|rule| Permission {
        subject: Some(rule.subject().clone()),
        object: rule.object().clone(),
        action: rule.action().to_string(),
    });
    Ok(Json(permissions))
}

pub async fn add_role_permission(
    enforcer: Data<RwLock<Enforcer>>,
    scope: Scope,
    current_user: CurrentUser,
    path: Path<RolePathParam>,
    permission: Json<Permission>,
) -> ApiResult<Json<Permission>> {
    Scope::from(vec!["roles", "permissions"]).matches_exactly(&scope)?;
    let role = &path.role;
    let enforcer = enforcer.read().unwrap();
    let sub = Guid::partitioned("role", role);
    enforcer.check(current_user.id(), &sub, "manage_permissions")?;

    let mut permission = permission;
    permission.subject = Some(sub.clone());
    enforcer.add_permission(sub, permission.object.clone(), &permission.action).await?;

    Ok(permission)
}

pub async fn remove_role_permission(
    enforcer: Data<RwLock<Enforcer>>,
    scope: Scope,
    current_user: CurrentUser,
    path: Path<RolePathParam>,
    permission: Json<Permission>,
) -> ApiResult<Json<Permission>> {
    Scope::from(vec!["roles", "permissions"]).matches_exactly(&scope)?;
    let role = &path.role;
    let enforcer = enforcer.read().unwrap();
    let sub = &Guid::partitioned("role", role);
    enforcer.check(current_user.id(), sub, "manage_permissions")?;

    let mut permission = permission;
    permission.subject = Some(sub.clone());
    enforcer.remove_permission(sub, permission.object.clone(), &permission.action).await?;

    Ok(permission)
}