use std::str::FromStr;

use config::{Config, File, Value};
use serde::Deserialize;
use slog::Level;
use structopt::StructOpt;

pub use log::LogFormat;

use crate::config::cli::Opts;
use crate::config::grpc::Grpc;
use crate::config::log::*;
use crate::error::Error;

mod cli;
mod grpc;
mod log;
mod tls;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    log: Log,
    grpc: Grpc,
}

impl Configuration {
    pub fn new() -> Result<Self, Error> {
        let opts: Opts = Opts::from_args();
        let mut cfg = Config::new();

        cfg.merge(File::with_name("enseadas").required(false))?;

        // CLI Args
        opts.set_cfg(&mut cfg)?;

        // Default
        Log::set_defaults(&mut cfg)?;
        Grpc::set_defaults(&mut cfg)?;

        cfg.try_into().map_err(Error::from)
    }

    pub fn log(&self) -> &Log {
        &self.log
    }
    pub fn grpc(&self) -> &Grpc {
        &self.grpc
    }
}
