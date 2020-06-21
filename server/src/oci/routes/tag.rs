use std::cmp::{max, min};

use actix_web::web::{Data, Path, Query};
use actix_web::{get, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::couchdb::repository::Repository;
use crate::oci::entity::Repo;
use crate::oci::error::{Error, ErrorCode};
use crate::oci::routes::RepoPath;
use crate::oci::service::RepoService;
use crate::oci::Result;

#[derive(Debug, Serialize)]
pub struct TagList {
    name: String,
    tags: Vec<String>,
}

impl From<&Repo> for TagList {
    fn from(repo: &Repo) -> Self {
        Self {
            name: repo.full_name(),
            tags: repo.tags().clone(),
        }
    }
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
) -> Result<HttpResponse> {
    let group = &repo.group;
    let name = &repo.name;

    log::debug!("looking for repo {}/{}", group, name);
    let repo = repos
        .find(&Repo::build_id(group, name))
        .await?
        .ok_or_else(|| Error::from(ErrorCode::NameUnknown))?;

    let mut res = HttpResponse::Ok();
    let res = if let Some(page) = page {
        let n = page.n.map(|n| min(max(n, 1), 50)).unwrap_or(25);
        let last = page.last;
        let tags = repo.tags().clone();
        let tags = tags.into_iter().skip(max(last, 1) - 1).take(n).collect();
        let list = TagList {
            name: repo.full_name(),
            tags,
        };
        res.header(
            http::header::LINK,
            format!(
                "</v2/{}/{}/tags/list?n={}&last={}>; rel=\"next\"",
                group,
                name,
                n,
                last + n
            ),
        )
        .json(list)
    } else {
        res.json(TagList::from(&repo))
    };

    Ok(res)
}
