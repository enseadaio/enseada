use std::collections::{HashMap, HashSet};
use std::io::{Error, ErrorKind};
use std::iter::FromIterator;

use include_dir::Dir;

use couchdb::db::Database;
use couchdb::migrator::Migrator;
use couchdb::{Couch, Result};
use enseada::couchdb::repository::Entity;
use oauth::client::Client;
use oauth::persistence::client::ClientEntity;
use oauth::scope::Scope;
use users::User;

use crate::config::Configuration;

static MIGRATION_DIR: Dir = include_dir!("./migrations");

fn map_couch_err<E: std::error::Error>(err: E) -> Error {
    Error::new(ErrorKind::Other, err.to_string())
}

pub async fn migrate(cfg: &Configuration) -> std::io::Result<()> {
    let couch = &crate::couchdb::from_config(cfg);
    couch.status().await.map_err(map_couch_err)?;

    run(couch, cfg).await.map_err(map_couch_err)
}

async fn run(couch: &Couch, cfg: &Configuration) -> Result<()> {
    log::info!("Running CouchDB migrations");
    let migs: Vec<String> = MIGRATION_DIR
        .files()
        .iter()
        .filter_map(map_file_with_ext("json"))
        .collect();

    let scripts = if let Some(scripts_dir) = MIGRATION_DIR.get_dir("scripts") {
        let files = scripts_dir.files();
        files.iter().filter(filter_file_with_ext("js")).fold(
            HashMap::with_capacity(files.len()),
            |mut map, file| {
                if let Some(content) = file.contents_utf8() {
                    map.insert(
                        file.path()
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                        content.to_string(),
                    );
                }
                map
            },
        )
    } else {
        HashMap::new()
    };

    let migrator = Migrator::new(couch, migs, scripts)?;
    migrator.run().await?;

    let oauth_db = couch.database(crate::couchdb::name::OAUTH, true);
    let users_db = couch.database(crate::couchdb::name::USERS, true);

    let public_host = cfg.public_url();
    create_oauth_client(
        &oauth_db,
        Client::public(
            "enseada".to_string(),
            Scope::from("*"),
            HashSet::from_iter(vec![public_host.join("/dashboard/auth/callback").unwrap()]),
        ),
    )
    .await?;

    create_oauth_client(
        &oauth_db,
        Client::public(
            "enseada-docs".to_string(),
            Scope::from("*"),
            HashSet::from_iter(vec![public_host.join("/api/docs/oauth-redirect").unwrap()]),
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

fn filter_file_with_ext(ext: &'static str) -> Box<dyn FnMut(&&include_dir::File) -> bool> {
    Box::new(move |file: &&include_dir::File| {
        if let Some(file_ext) = file.path().extension() {
            ext == file_ext
        } else {
            false
        }
    })
}

fn map_file_with_ext(ext: &'static str) -> Box<dyn FnMut(&include_dir::File) -> Option<String>> {
    Box::new(move |file: &include_dir::File| {
        if let Some(file_ext) = file.path().extension() {
            if ext == file_ext {
                file.contents_utf8().map(str::to_string)
            } else {
                None
            }
        } else {
            None
        }
    })
}
