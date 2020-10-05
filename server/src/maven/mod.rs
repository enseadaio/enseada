use std::sync::{Arc, RwLock};

use actix_web::web::ServiceConfig;
use serde::Deserialize;

use couchdb::db::Database;
use enseada::storage::Provider;
use events::EventBus;
use maven::service::RepoService;

mod api;
mod files;

pub fn mount(
    db: Database,
    bus: Arc<RwLock<EventBus>>,
    store: Arc<Provider>,
) -> Box<impl FnOnce(&mut ServiceConfig)> {
    Box::new(move |cfg: &mut ServiceConfig| {
        let repo = RepoService::new(db, bus, store);
        cfg.data(repo);

        cfg.service(api::list_repos);
        cfg.service(api::create_repo);
        cfg.service(api::get_repo);
        cfg.service(api::get_repo_files);
        cfg.service(api::delete_repo);

        cfg.service(files::get);
        cfg.service(files::put);
    })
}
