#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_derive;

mod config;
mod couchdb;
mod errors;
mod handlers;
mod logger;
mod responses;
mod routes;
mod server;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    logger::init();

    log::info!("Starting Enseada...");

    server::run().await
}
