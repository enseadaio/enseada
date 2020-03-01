#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use crate::config::Configuration;

mod logger;
mod config;
mod server;
mod routes;
mod handlers;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let cfg = &Configuration::new().unwrap();
    logger::init(cfg);
    server::run().await
}
