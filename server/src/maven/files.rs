use std::sync::Arc;

use actix_web::web::{Bytes, Data, Path};
use actix_web::{get, put, HttpResponse};
use http::StatusCode;
use tokio::sync::RwLock;

use enseada::couchdb::repository::Entity;
use enseada::error::Error;
use maven::entity::Repo;
use maven::file::{parse_file_path, File};
use maven::service::RepoService;
use oauth::scope::Scope;
use rbac::Enforcer;

use crate::http::error::ApiError;
use crate::http::extractor::scope::OAuthScope;
use crate::http::extractor::user::CurrentUser;
use crate::http::ApiResult;

#[get("/maven/{tail:.*}")]
pub async fn get(
    repos: Data<RepoService>,
    Path(location): Path<String>,
    current_user: Option<CurrentUser>,
    scope: Option<OAuthScope>,
    enforcer: Data<Arc<RwLock<Enforcer>>>,
) -> ApiResult<HttpResponse> {
    let file_pointer = parse_file_path(&location)
        .ok_or_else(|| ApiError::invalid(format!("{} is not a valid Maven path", location)))?;

    let repo = repos.find_by_location(file_pointer.prefix()).await?;
    let repo = match repo {
        Some(repo) => repo,
        None => return Err(Error::not_found("Maven repository", file_pointer.prefix()).into()),
    };

    if repo.is_private() {
        if let Some((current_user, scope)) = Option::zip(current_user, scope) {
            Scope::from("maven:repos:pull").matches(&scope)?;
            let enforcer = enforcer.read().await;
            enforcer.check(
                current_user.id(),
                &Repo::build_guid(file_pointer.prefix()),
                "pull",
            )?;
        }
    }

    let file = repos
        .get_file(&repo, file_pointer.version(), file_pointer.filename())
        .await?;
    Ok(HttpResponse::Ok()
        .content_type("application/octet-stream")
        .streaming(file.into_byte_stream()))
}

#[put("/maven/{tail:.*}")]
pub async fn put(
    repos: Data<RepoService>,
    Path(location): Path<String>,
    body: Bytes,
    current_user: Option<CurrentUser>,
    scope: Option<OAuthScope>,
    enforcer: Data<Arc<RwLock<Enforcer>>>,
) -> ApiResult<HttpResponse> {
    let file_pointer = parse_file_path(&location)
        .ok_or_else(|| ApiError::invalid(format!("{} is not a valid Maven path", location)))?;

    let repo = repos.find_by_location(file_pointer.prefix()).await?;
    let repo = match repo {
        Some(repo) => repo,
        None => return Err(Error::not_found("Maven repository", file_pointer.prefix()).into()),
    };

    if repo.is_private() {
        if let Some((current_user, scope)) = Option::zip(current_user, scope) {
            Scope::from("maven:repos:push").matches(&scope)?;
            let enforcer = enforcer.read().await;
            enforcer.check(
                current_user.id(),
                &Repo::build_guid(file_pointer.prefix()),
                "push",
            )?;
        }
    }

    let filename = file_pointer.filename();
    if let Some(version) = file_pointer.version() {
        let file_exists = repos.is_file_present(&repo, version, filename).await?;
        if !version.is_snapshot() && file_exists {
            return Err(ApiError::new(
                StatusCode::CONFLICT,
                format!(
                    "could not update file {}, version {} is immutable",
                    filename, version
                ),
            ));
        }
    }

    let file = File::from_bytes(file_pointer.version(), filename, body);
    repos.store_file(&repo, file).await?;

    Ok(HttpResponse::Accepted().finish())
}
