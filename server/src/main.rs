#[macro_use]
extern crate include_dir;
#[macro_use]
extern crate lazy_static;

mod config;
mod couchdb;
mod http;
mod logger;
mod oauth;
mod observability;
mod rbac;
mod responses;
mod routes;
mod server;
mod templates;
mod ui;
mod user;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    logger::init();

    couchdb::migrate().await?;

    log::info!("Starting Enseada...");

    server::run().await?;

    log::info!("Stopping Enseada...");

    Ok(())
}
