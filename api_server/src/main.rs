use actix::Arbiter;
use futures::TryFutureExt;
use url::Url;

use couchdb::Couch;

use crate::config::Configuration;
use crate::error::Error;

mod config;
mod error;
mod http;
mod logger;

type ServerResult = Result<(), Error>;

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = &Configuration::new()?;
    let logger = logger::create_logger(cfg);
    slog::debug!(logger, "Config: {:?}", cfg);

    let couch = &Couch::new(Url::parse("http://127.0.0.1:5984").unwrap(), "enseada".to_string(), "enseada".to_string());

    let controller_arbiter = Arbiter::new().handle();

    slog::info!(logger, "Starting API server");
    tokio::try_join!(
        http::start(logger.new(slog::o!("server" => "http")), couch.clone()),
        users::start(logger.clone(), couch.clone(), &controller_arbiter).map_err(Error::from)
    )?;

    Ok(())
}
