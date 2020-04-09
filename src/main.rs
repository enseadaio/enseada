#[macro_use]
extern crate lazy_static;

mod config;
mod couchdb;
mod errors;
mod handlers;
mod logger;
mod oauth;
mod responses;
mod routes;
mod secure;
mod server;
mod templates;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    logger::init();

    couchdb::migrate().await?;

    log::info!("Starting Enseada...");

    server::run().await?;

    log::info!("Stopping Enseada...");

    Ok(())
}
