use std::io::{Error, ErrorKind, Result};

use crate::couchdb::Couch;
use crate::couchdb::db::{self, Database};
use crate::oauth::client::Client;
use crate::oauth::scope::Scope;
use crate::oauth::persistence::client::ClientEntity;
use std::collections::HashSet;
use std::iter::FromIterator;
use url::Url;

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
        return Ok(true);
    }

    db.create_self().await
}

async fn create_oauth_client(db: &Database, client: Client) -> reqwest::Result<bool> {
    log::debug!("Creating oauth client");
    let guid = ClientEntity::build_guid(client.client_id());
    if db.exists(guid.to_string().as_str()).await? {
        log::debug!("Client {}, already exists. Skipping", &guid);
        return Ok(true);
    }

    let entity = ClientEntity::from(client);
    db.put::<&ClientEntity, serde_json::Value>(entity.id().to_string().as_str(), &entity).await
        .map(|_| true)
}