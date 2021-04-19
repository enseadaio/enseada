use actix::Arbiter;
use slog::Logger;
use url::Url;

use couchdb::Couch;
use couchdb::db::Database;

use crate::config::Configuration;
use crate::resources::{ResourceManager, Watcher};

mod config;
mod controllers;
mod error;
mod grpc;
mod http;
mod logger;
mod resources;
mod tls;

type ServerResult = Result<(), crate::error::Error>;

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = &Configuration::new()?;
    let logger = logger::create_logger(cfg);
    slog::debug!(logger, "Config: {:?}", cfg);

    let couch = &Couch::new(Url::parse("http://127.0.0.1:5984").unwrap(), "enseada".to_string(), "enseada".to_string());

    let controller_arbiter = Arbiter::new().handle();

    slog::info!(logger, "Starting API server");
    tokio::try_join!(
        http::start(logger.new(slog::o!("server" => "http"))),
        grpc::start(cfg.clone(), logger.new(slog::o!("server" => "grpc")), couch),
        controllers::users::start(logger.new(slog::o!("controller" => "user")), couch.database("users", true), &controller_arbiter)
    )?;

    Ok(())
}
