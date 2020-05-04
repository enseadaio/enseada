use std::collections::HashSet;
use std::io::{Error, ErrorKind};
use std::iter::FromIterator;

use crate::config::{CONFIG, Configuration};
use crate::couchdb;
use crate::couchdb::Couch;
use crate::couchdb::db::{self, Database};
use crate::couchdb::Result;
use crate::oauth::client::Client;
use crate::oauth::persistence::client::ClientEntity;
use crate::oauth::scope::Scope;
use crate::user::User;

pub async fn migrate() -> std::io::Result<()> {
    let couch = &couchdb::SINGLETON;
    run(couch, &CONFIG).await.map_err(|err| Error::new(ErrorKind::Other, err.to_string()))
}

async fn run(couch: &Couch, cfg: &Configuration) -> Result<()> {
    log::info!("Running CouchDB migrations");

    let oauth_db = couch.database(db::name::OAUTH, true);
    create_db_if_not_exist(&oauth_db).await?;

    let users_db = couch.database(db::name::USERS, true);
    create_db_if_not_exist(&users_db).await?;

    let rbac_db = couch.database(db::name::RBAC, true);
    create_db_if_not_exist(&rbac_db).await?;

    let public_host = cfg.public_host();
    create_oauth_client(&oauth_db, Client::public(
        "enseada".to_string(),
        Scope::from("*"),
        HashSet::from_iter(vec![public_host.join("/ui/auth/callback").unwrap()]))).await?;

    let root_pwd = cfg.root_password();
    create_root_user(&users_db, root_pwd).await?;

    log::info!("Migrations completed");
    Ok(())
}

async fn create_db_if_not_exist(db: &Database) -> Result<bool> {
    log::debug!("Creating database {}", db.name());
    if db.get_self().await.is_ok() {
        log::debug!("Database {} already exists. Skipping", db.name());
        return Ok(true);
    }

    db.create_self().await
}

async fn create_oauth_client(db: &Database, client: Client) -> Result<bool> {
    log::debug!("Creating oauth client");
    let guid = ClientEntity::build_guid(client.client_id());
    if db.exists(&guid.to_string()).await? {
        log::debug!("Client {} already exists. Skipping", &guid);
        return Ok(true);
    }

    let entity = ClientEntity::from(client);
    db.put(&entity.id().to_string(), &entity).await
        .map(|res| res.ok)
}

async fn create_root_user(db: &Database, password: String) -> Result<bool> {
    log::debug!("Creating root user");
    let user = User::new(String::from("root"), password).unwrap();
    if db.exists(&user.id().to_string()).await? {
        log::debug!("Root user already exists. Skipping");
        return Ok(true);
    }

    db.put(&user.id().to_string(), user).await
        .map(|res| res.ok)
}