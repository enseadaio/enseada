#[macro_use]
extern crate lazy_static;

mod config;
mod couchdb;
mod containers;
mod error;
mod guid;
mod http;
mod logger;
mod oauth;
mod pagination;
mod rbac;
mod responses;
mod routes;
mod secure;
mod server;
mod storage;
mod templates;
mod user;

async fn run() -> std::io::Result<()> {
    logger::init();

    couchdb::migrate().await?;

    log::info!("Starting Enseada...");

    server::run().await?;

    log::info!("Stopping Enseada...");

    Ok(())
}

#[actix_rt::main]
async fn main() {
    if let Err(err) = run().await {
        log::error!("{}", err);
    }
}
