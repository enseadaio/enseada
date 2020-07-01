use std::str::FromStr;

use log::{Level, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::json::JsonEncoder;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::encode::Encode;

use crate::config::Configuration;

fn encoder(fmt: &str) -> Box<dyn Encode> {
    match fmt {
        "json" => Box::new(JsonEncoder::new()),
        _ => Box::new(PatternEncoder::default()),
    }
}

fn level(lvl: &str) -> LevelFilter {
    Level::from_str(lvl.to_lowercase().as_str())
        .unwrap_or(Level::Info)
        .to_level_filter()
}

pub fn init(cfg: &Configuration) {
    let fmt = cfg.log().format();
    let lvl = &cfg.log().level();
    let root_lvl = &cfg.log().root_level();

    let stdout = ConsoleAppender::builder().encoder(encoder(&fmt)).build();

    match Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .logger(primary(lvl))
        .logger(lib(lvl))
        .logger(couchdb(lvl))
        .logger(tracing(cfg))
        .build(Root::builder().appender("stdout").build(level(root_lvl)))
    {
        Ok(config) => {
            if let Err(error) = log4rs::init_config(config) {
                panic!("{}", error);
            }
        }
        Err(error) => panic!("{}", error),
    }
}

fn primary(lvl: &str) -> Logger {
    Logger::builder().build("enseada_server", level(lvl))
}

fn lib(lvl: &str) -> Logger {
    Logger::builder().build("enseada", level(lvl))
}

fn couchdb(lvl: &str) -> Logger {
    Logger::builder().build("couchdb", level(lvl))
}

fn tracing(cfg: &Configuration) -> Logger {
    if cfg.tracing().log() {
        Logger::builder().build("tracing", LevelFilter::Info)
    } else {
        Logger::builder().build("tracing", LevelFilter::Off)
    }
}
