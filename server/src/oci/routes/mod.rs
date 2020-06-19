use actix_web::web::ServiceConfig;

use crate::oci::service::RepoService;

mod api;

pub fn mount(cfg: &mut ServiceConfig) {
    let couch = &crate::couchdb::SINGLETON;
    let db = couch.database(crate::couchdb::name::OCI, true);
    let repo = RepoService::new(db);
    cfg.data(repo);

    cfg.service(api::list_repos);
    cfg.service(api::create_repo);
    cfg.service(api::get_repo);
    cfg.service(api::delete_repo);
}
