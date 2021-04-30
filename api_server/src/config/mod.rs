use config::{Config, File};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use crate::config::cli::Opts;
use crate::config::controllers::Controllers;
use crate::config::db::CouchDB;
use crate::config::gc::GarbageCollection;
pub use crate::config::http::Http;
pub use crate::config::log::*;
use crate::error::Error;

mod cli;
mod controllers;
mod db;
mod gc;
mod http;
mod log;
pub mod tls;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Configuration {
    log: Log,
    http: Http,
    couchdb: CouchDB,
    controllers: Controllers,
    gc: GarbageCollection,
}

impl Configuration {
    pub fn new() -> Result<Self, Error> {
        let opts: Opts = Opts::from_args();
        let mut cfg = Config::new();

        cfg.merge(File::with_name("enseada").required(false))?;

        // CLI Args
        opts.set_cfg(&mut cfg)?;

        // Default
        Log::set_defaults(&mut cfg)?;
        Http::set_defaults(&mut cfg)?;
        CouchDB::set_defaults(&mut cfg)?;
        Controllers::set_defaults(&mut cfg)?;
        GarbageCollection::set_defaults(&mut cfg)?;

        cfg.try_into().map_err(Error::from)
    }

    pub fn log(&self) -> &Log {
        &self.log
    }

    pub fn http(&self) -> &Http {
        &self.http
    }

    pub fn couchdb(&self) -> &CouchDB {
        &self.couchdb
    }

    pub fn controllers(&self) -> &Controllers {
        &self.controllers
    }

    pub fn gc(&self) -> &GarbageCollection {
        &self.gc
    }

    pub fn pretty_print(&self) -> String {
        match self.log.format() {
            LogFormat::Text => serde_json::to_string_pretty(&self),
            LogFormat::Json => serde_json::to_string(&self),
        }.expect("failed to pretty print configuration")
    }
}
