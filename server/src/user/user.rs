use std::ops::Deref;
use std::sync::Arc;

use actix_web::web::{Data, Json, Path, Query, ServiceConfig};
use actix_web::{delete, get, post, put, HttpResponse};
use http::StatusCode;
use tokio::sync::RwLock;

use api::users::v1beta1::{UserModel, UserPost, UserPut};
use enseada::couchdb::repository::{Entity, Repository};
use enseada::guid::Guid;
use enseada::pagination::Page;
use oauth::scope::Scope;
use rbac::Enforcer;
use users::{User, UserService};

use crate::http::error::ApiError;
use crate::http::extractor::scope::OAuthScope;
use crate::http::extractor::user::CurrentUser;
use crate::http::responses;
use crate::http::{ApiResult, PaginationQuery};
use crate::user::{map_owned_user, map_user, UsernamePathParam};

#[get("/api/v1beta1/users")]
pub async fn list(
    service: Data<UserService>,
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    list: Query<PaginationQuery>,
) -> ApiResult<Json<Page<UserModel>>> {
    Scope::from("users:read").matches(&scope)?;
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &Guid::simple("users"), "read")?;
    let limit = list.limit();
    let offset = list.offset();

    log::info!(
        "Listing users with limit {} and offset {:?}",
        &limit,
        &offset
    );

    let page = service.list(limit, offset).await?.map(map_owned_user);
    Ok(Json(page))
}

#[post("/api/v1beta1/users")]
pub async fn register(
    service: Data<UserService>,
    data: Json<UserPost>,
    scope: OAuthScope,
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    current_user: CurrentUser,
) -> Result<Json<UserModel>, ApiError> {
    Scope::from("users:manage").matches(&scope)?;
    let enf = enforcer.read().await;
    enf.check(current_user.id(), &Guid::simple("users"), "create")?;

    let user = User::new(data.username.clone(), data.password.clone())?;
    let user = service.save(user).await?;

    if !data.roles.is_empty() {
        // We exclusively lock the enforcer to avoid having
        // the internal model being updated for every role insert
        // TODO: support bulk put in CouchDB to have them inserted all at once so that we don't need exclusive access
        // let enf = enforcer.write().unwrap();
        for role in &data.roles {
            enf.add_role_to_principal(user.id().clone(), role).await?;
        }
    }

    responses::ok(map_user(&user))
}

#[get("/api/v1beta1/users/{username}")]
pub async fn get(
    service: Data<UserService>,
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    path: Path<UsernamePathParam>,
) -> ApiResult<Json<UserModel>> {
    Scope::from("users:read").matches(&scope)?;
    let username = &path.username;
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &User::build_guid(username), "read")?;
    service
        .find(username)
        .await?
        .ok_or_else(|| ApiError::not_found(format!("User '{}' not found", username)))
        .map(map_owned_user)
        .map(Json)
}

#[put("/api/v1beta1/users/{username}")]
pub async fn update(
    service: Data<UserService>,
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    path: Path<UsernamePathParam>,
    data: Json<UserPut>,
) -> ApiResult<Json<UserModel>> {
    Scope::from("users:manage").matches(&scope)?;
    let username = &path.username;
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &User::build_guid(username), "update")?;

    log::debug!("looking up user {}", username);
    let mut user = service
        .find(username)
        .await?
        .ok_or_else(|| ApiError::not_found(format!("User '{}' not found", username)))?;

    let mut dirty = false;

    log::debug!("updating user {}", username);
    if let Some(enabled) = data.enabled {
        if user.id() == current_user.id() {
            return Err(ApiError::invalid("users cannot disable themselves"));
        }
        enforcer.check(current_user.id(), &User::build_guid(username), "disable")?;
        if enabled != user.is_enabled() {
            log::debug!("setting enabled to '{}' for user {}", enabled, username);
            user.set_enabled(enabled);
            dirty = true;
        }
    }

    if dirty {
        log::debug!("saving user {}: {:?}", username, &user);
        user = service.save(user).await?;
    }

    Ok(Json(map_user(&user)))
}

#[delete("/api/v1beta1/users/{username}")]
pub async fn delete(
    service: Data<UserService>,
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
    path: Path<UsernamePathParam>,
) -> ApiResult<Json<UserModel>> {
    Scope::from("users:manage").matches(&scope)?;
    let username = &path.username;
    if username == "root" {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "root user cannot be deleted",
        ));
    }
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &User::build_guid(username), "delete")?;

    let user = service
        .find(username)
        .await?
        .ok_or_else(|| ApiError::not_found(format!("User '{}' not found", username)))?;

    service.delete(&user).await?;
    Ok(Json(map_user(&user)))
}

#[get("/api/v1beta1/users/me")]
pub async fn me(current_user: CurrentUser, scope: OAuthScope) -> ApiResult<Json<UserModel>> {
    Scope::from("profile").matches(&scope)?;
    Ok(Json(map_user(&current_user.deref())))
}
