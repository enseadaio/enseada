#[macro_use]
extern crate lazy_static;

use actix::Arbiter;
use futures::TryFutureExt;
use slog::Logger;

use controller_runtime::GarbageCollector;
use couchdb::Couch;

use crate::config::Configuration;
use crate::error::Error;

mod config;
mod error;
mod http;
mod logger;
mod telemetry;

type ServerResult = Result<(), Error>;

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = &Configuration::new()?;
    let logger = logger::create_logger(cfg);
    slog::debug!(logger, ""; "config" => cfg.pretty_print());

    let couch_cfg = cfg.couchdb();
    let couch = &Couch::new(
        couch_cfg.url().clone(),
        couch_cfg.username().to_string(),
        couch_cfg.password().to_string(),
    );

    let controller_arbiter = Arbiter::new().handle();

    let enforcer = acl::api::v1alpha1::create_enforcer(logger.clone(), couch.clone());

    start_gc(logger.clone(), &couch, cfg);

    slog::info!(logger, "Starting API server");
    tokio::try_join!(
        http::start(logger.new(slog::o!("process" => "http")), couch.clone(), cfg, enforcer.clone()),
        acl::api::v1alpha1::start_controllers(logger.clone(), couch.clone(), &controller_arbiter, cfg.controllers().get("acl").polling_interval(), enforcer).map_err(Error::from),
        auth::api::v1alpha1::start_controllers(logger.clone(), couch.clone(), &controller_arbiter, cfg.controllers().get("auth").polling_interval()).map_err(Error::from),
    )?;

    Ok(())
}

fn start_gc(logger: Logger, couch: &Couch, cfg: &Configuration) {
    slog::info!(logger, "Starting garbage collection");

    let arbiter = Arbiter::new().handle();

    GarbageCollector::start(logger.new(slog::o!("process" => "gc", "group" => api::core::API_GROUP.clone())), couch.database(&api::core::API_GROUP, true), &arbiter, cfg.gc().polling_interval());
    GarbageCollector::start(logger.new(slog::o!("process" => "gc", "group" => acl::api::API_GROUP.clone())), couch.database(&acl::api::API_GROUP, true), &arbiter, cfg.gc().polling_interval());
    GarbageCollector::start(logger.new(slog::o!("process" => "gc", "group" => auth::api::API_GROUP.clone())), couch.database(&auth::api::API_GROUP, true), &arbiter, cfg.gc().polling_interval());
}
