use std::collections::HashSet;
use std::io::{Error, ErrorKind};
use std::iter::FromIterator;

use include_dir::{Dir, File};

use couchdb::db::Database;
use couchdb::migrator::Migrator;
use couchdb::{Couch, Result};

use crate::config::{Configuration, CONFIG};
use crate::oauth::client::Client;
use crate::oauth::persistence::client::ClientEntity;
use crate::oauth::scope::Scope;
use crate::user::User;

static MIGRATION_DIR: Dir = include_dir!("./migrations");

pub async fn migrate() -> std::io::Result<()> {
    let couch = &crate::couchdb::SINGLETON;

    run(couch, &CONFIG)
        .await
        .map_err(|err| Error::new(ErrorKind::Other, err.to_string()))
}

async fn run(couch: &Couch, cfg: &'static Configuration) -> Result<()> {
    log::info!("Running CouchDB migrations");
    let migs: Vec<String> = MIGRATION_DIR
        .files()
        .iter()
        .map(File::contents_utf8)
        .filter(Option::is_some)
        .map(Option::unwrap)
        .map(str::to_string)
        .collect();

    let migrator = Migrator::new(couch, migs)?;
    migrator.run().await?;

    let oauth_db = couch.database(crate::couchdb::name::OAUTH, true);
    let users_db = couch.database(crate::couchdb::name::USERS, true);

    let public_host = cfg.public_host();
    create_oauth_client(
        &oauth_db,
        Client::public(
            "enseada".to_string(),
            Scope::from("*"),
            HashSet::from_iter(vec![public_host.join("/ui/auth/callback").unwrap()]),
        ),
    )
    .await?;

    let root_pwd = cfg.root_password();
    create_root_user(&users_db, root_pwd).await?;

    log::info!("Migrations completed");
    Ok(())
}

async fn create_oauth_client(db: &Database, client: Client) -> Result<()> {
    log::debug!("Creating oauth client");
    let guid = ClientEntity::build_guid(client.client_id());
    if db.exists(&guid.to_string()).await? {
        log::debug!("Client {} already exists. Skipping", &guid);
        return Ok(());
    }

    let entity = ClientEntity::from(client);
    db.put(&entity.id().to_string(), &entity).await.map(|_| ())
}

async fn create_root_user(db: &Database, password: String) -> Result<()> {
    log::debug!("Creating root user");
    let user = User::new(String::from("root"), password).unwrap();
    if db.exists(&user.id().to_string()).await? {
        log::debug!("Root user already exists. Skipping");
        return Ok(());
    }

    db.put(&user.id().to_string(), user).await.map(|_| ())
}
