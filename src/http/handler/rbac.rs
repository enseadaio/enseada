use actix_web::web::{Data, Json, Path, Query};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::guid::Guid;
use crate::http::error::ApiError;
use crate::http::extractor::user::CurrentUser;
use crate::http::handler::{ApiResult, PaginationQuery};
use crate::http::handler::user::UsernamePathParam;
use crate::oauth::scope::Scope;
use crate::pagination::{Cursor, Page};
use crate::rbac::{Enforcer, Rule};
use crate::user::UserService;

#[derive(Debug, Serialize, PartialEq)]
pub struct RoleResponse {
    pub role: String,
}

pub async fn get_user_roles(
    service: Data<UserService>,
    enforcer: Data<RwLock<Enforcer>>,
    scope: Scope,
    current_user: CurrentUser,
    path: Path<UsernamePathParam>,
    list: Query<PaginationQuery>,
) -> ApiResult<Json<Page<String>>> {
    Scope::from(vec!["users:read", "roles"]).matches_exactly(&scope)?;
    let username = &path.username;
    let enforcer = enforcer.read().await;
    let sub = Guid::partitioned("user", username);
    enforcer.check(current_user.id(), &sub, "read_roles")?;

    if service.find_user(username).await?.is_none() {
        return Err(ApiError::NotFound(format!("User {} not found", username)));
    }

    let limit = list.limit();
    let cursor = list.cursor();

    let cursor = if let Some(cursor) = cursor {
        Some(Cursor::from_b64(cursor)?)
    } else {
        None
    };

    let page = enforcer.list_principal_roles(&sub, limit, cursor.as_ref()).await?;
    Ok(Json(page))
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
    let enforcer = enforcer.read().await;
    let username = &path.username;
    let sub = Guid::partitioned("user", username);
    enforcer.check(current_user.id(), &sub, "manage_roles")?;

    if service.find_user(username).await?.is_none() {
        return Err(ApiError::NotFound(format!("User {} not found", username)));
    }

    let role = &path.role;
    enforcer.add_role_to_principal(sub.clone(), role).await?;
    Ok(Json(RoleResponse {
        role: role.clone(),
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
    let enforcer = enforcer.read().await;
    let username = &path.username;
    let sub = &Guid::partitioned("user", username);
    enforcer.check(current_user.id(), sub, "manage_roles")?;

    if service.find_user(username).await?.is_none() {
        return Err(ApiError::NotFound(format!("User {} not found", username)));
    }

    let role = &path.role;
    enforcer.remove_role_from_principal(sub, role).await?;
    Ok(Json(RoleResponse {
        role: role.clone(),
    }))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Permission {
    #[serde(skip_serializing_if = "Option::is_none")]
    subject: Option<Guid>,
    object: Guid,
    action: String,
}

impl From<&Rule> for Permission {
    fn from(rule: &Rule) -> Self {
        Permission {
            subject: Some(rule.subject().clone()),
            object: rule.object().clone(),
            action: rule.action().to_string(),
        }
    }
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
    let enforcer = enforcer.read().await;
    let sub = &Guid::partitioned("user", username);
    enforcer.check(current_user.id(), sub, "read_permissions")?;

    let limit = list.limit();
    let cursor = list.cursor();

    let cursor = if let Some(cursor) = cursor {
        Some(Cursor::from_b64(cursor)?)
    } else {
        None
    };

    let page = enforcer.list_principal_permissions(&sub, limit, cursor.as_ref()).await?;
    let permissions = page.map(|rule| Permission::from(rule));
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
    let enforcer = enforcer.read().await;
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
    let enforcer = enforcer.read().await;
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
    let enforcer = enforcer.read().await;
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
    let permissions = rules.map(|rule| Permission::from(rule));
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
    let enforcer = enforcer.read().await;
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
    let enforcer = enforcer.read().await;
    let sub = &Guid::partitioned("role", role);
    enforcer.check(current_user.id(), sub, "manage_permissions")?;

    let mut permission = permission;
    permission.subject = Some(sub.clone());
    enforcer.remove_permission(sub, permission.object.clone(), &permission.action).await?;

    Ok(permission)
}