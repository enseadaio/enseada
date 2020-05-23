use actix_web::HttpResponse;
use actix_web::web::{Data, Json, Path, Query, ServiceConfig};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::couchdb;
use crate::couchdb::db;
use crate::guid::Guid;
use crate::http::error::ApiError;
use crate::http::extractor::{scope::Scope, user::CurrentUser};
use crate::http::handler::{ApiResult, PaginationQuery};
use crate::pagination::{Cursor, Page};
use crate::rbac::Enforcer;
use crate::responses;
use crate::user::{User, UserService};

pub fn add_user_service(app: &mut ServiceConfig) {
    let couch = &couchdb::SINGLETON;
    let db = couch.database(db::name::USERS, true);
    let service = UserService::new(db);
    app.data(service);
}

#[derive(Debug, Serialize, PartialEq)]
pub struct UserResponse {
    pub username: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse { username: user.username().to_string() }
    }
}

impl From<&User> for UserResponse {
    fn from(user: &User) -> Self {
        UserResponse { username: user.username().to_string() }
    }
}

pub async fn list(
    service: Data<UserService>,
    enforcer: Data<RwLock<Enforcer>>,
    scope: Scope,
    current_user: CurrentUser,
    list: Query<PaginationQuery>,
) -> ApiResult<Json<Page<UserResponse>>> {
    Scope::from("users:read").matches(&scope)?;
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &Guid::simple("users"), "read")?;
    let limit = list.limit();
    let cursor = list.cursor();

    log::info!("Listing users with limit {} and cursor {:?}", &limit, &cursor);

    let cursor = if let Some(cursor) = cursor {
        Some(Cursor::from_b64(cursor)?)
    } else {
        None
    };
    let page = service.list_users(limit, cursor.as_ref()).await?.map(|user| UserResponse::from(user));
    Ok(Json(page))
}

#[derive(Debug, Deserialize)]
pub struct UsernamePathParam {
    pub username: String,
}

pub async fn get(
    service: Data<UserService>,
    enforcer: Data<RwLock<Enforcer>>,
    scope: Scope,
    current_user: CurrentUser,
    path: Path<UsernamePathParam>,
) -> ApiResult<Json<UserResponse>> {
    Scope::from("users:read").matches(&scope)?;
    let username = &path.username;
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &User::build_guid(username), "read")?;
    service.find_user(username).await?
        .ok_or_else(|| ApiError::NotFound(format!("User {} not found", username)))
        .map(UserResponse::from)
        .map(Json)
}

pub async fn delete(
    service: Data<UserService>,
    scope: Scope,
    path: Path<UsernamePathParam>,
) -> ApiResult<HttpResponse> {
    Scope::from("users:manage").matches(&scope)?;
    let username = &path.username;
    let user = service.find_user(username).await?
        .ok_or_else(|| ApiError::NotFound(username.clone()))?;

    service.delete_user(&user).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn me(
    user: CurrentUser,
    scope: Scope,
) -> ApiResult<Json<UserResponse>> {
    Scope::from("profile").matches(&scope)?;
    Ok(Json(user.into()))
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Registration {
    pub username: String,
    pub password: String,
    pub roles: Option<Vec<String>>,
}

pub async fn register(
    service: Data<UserService>,
    data: Json<Registration>,
    scope: Scope,
    enforcer: Data<RwLock<Enforcer>>,
    current_user: CurrentUser,
) -> Result<Json<UserResponse>, ApiError> {
    Scope::from("users:manage").matches(&scope)?;
    let enf = enforcer.read().await;
    enf.check(current_user.id(), &Guid::simple("users"), "create")?;

    let user = User::new(data.username.clone(), data.password.clone())?;
    let user = service.save_user(user).await?;

    if let Some(roles) = &data.roles {
        // We exclusively lock the enforcer to avoid having
        // the internal model being updated for every role insert
        // TODO: support bulk put in CouchDB to have them inserted all at once so that we don't need exclusive access
        // let enf = enforcer.write().unwrap();
        for role in roles {
            enf.add_role_to_principal(user.id().clone(), role).await?;
        }
    }

    responses::ok(UserResponse { username: user.username().to_string() })
}
