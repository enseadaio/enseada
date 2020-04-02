use crate::couchdb::Couch;
use std::io::{Result, Error, ErrorKind};
use crate::couchdb::db::{self, Database};

pub async fn migrate() -> Result<()> {
    let couch = Couch::from_global_config();

    run(&couch).await.map_err(|err| Error::new(ErrorKind::Other, err.to_string()))
}

async fn run(couch: &Couch) -> reqwest::Result<()> {
    log::info!("Running CouchDB migrations");
    let oauth_db = couch.database(db::name::OAUTH, true);
    create_db_if_not_exist(&oauth_db).await?;

    log::info!("Migrations completed");
    Ok(())
}

async fn create_db_if_not_exist(db: &Database) -> reqwest::Result<bool> {
    log::debug!("Creating database {}", db.name());
    if let Ok(_) = db.get_self().await {
        log::debug!("Database {} already exists. Skipping", db.name());
        return Ok(true)
    }

    db.create_self().await
}