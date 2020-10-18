use std::cmp::{max, min};
use std::sync::Arc;

use actix_web::web::{Data, Path, Query};
use actix_web::{get, HttpResponse};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use enseada::couchdb::repository::{Entity, Repository};
use oauth::scope::Scope;
use oci::entity::Repo;
use oci::error::{Error, ErrorCode};
use oci::service::RepoService;
use rbac::Enforcer;

use crate::http::extractor::scope::OAuthScope;
use crate::http::extractor::user::CurrentUser;
use crate::oci::{RepoPath, Result};

#[derive(Debug, Serialize)]
pub struct TagList {
    name: String,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TagPagination {
    n: Option<usize>,
    last: usize,
}

#[get("/{group}/{name}/tags/list")]
pub async fn list(
    repos: Data<RepoService>,
    repo: Path<RepoPath>,
    page: Option<Query<TagPagination>>,
    enforcer: Data<Arc<RwLock<Enforcer>>>,
    scope: OAuthScope,
    current_user: CurrentUser,
) -> Result<HttpResponse> {
    Scope::from("oci:image:pull").matches(&scope)?;
    let group = &repo.group;
    let name = &repo.name;
    let repo_id = Repo::build_id(group, name);
    let enforcer = enforcer.read().await;
    enforcer.check(current_user.id(), &Repo::build_guid(&repo_id), "image:pull")?;

    log::debug!("looking for repo {}/{}", group, name);
    let repo = repos
        .find(&Repo::build_id(group, name))
        .await?
        .ok_or_else(|| Error::from(ErrorCode::NameUnknown))?;

    let mut res = HttpResponse::Ok();
    let res = if let Some(page) = page {
        let limit = page.n.map(|n| min(max(n, 1), 50)).unwrap_or(25);
        let offset = max(page.last, 1) - 1;
        let tags = repos.list_repo_tags(&repo, limit, offset).await?;
        let list = TagList {
            name: repo.full_name(),
            tags: tags.into_iter().collect(),
        };
        res.header(
            http::header::LINK,
            format!(
                "</v2/{}/{}/tags/list?n={}&last={}>; rel=\"next\"",
                group,
                name,
                limit,
                offset + limit
            ),
        )
        .json(list)
    } else {
        let tags = repos.list_all_repo_tags(&repo).await?;
        res.json(TagList {
            name: repo.full_name(),
            tags,
        })
    };

    Ok(res)
}
