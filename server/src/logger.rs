use std::str::FromStr;

use log::{Level, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::json::JsonEncoder;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::encode::Encode;

use crate::config::Configuration;

static MODULES: &'static [&'static str] = &[
    "couchdb",
    "enseada",
    "users",
    "observability",
    "rbac",
    "oci",
    "oauth",
    "api",
    "enseada_server",
];

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

    let mut builder = Config::builder();

    for module in MODULES {
        builder = builder.logger(logger(module, lvl))
    }

    match builder
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
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

fn logger(name: &str, lvl: &str) -> Logger {
    Logger::builder().build(name, level(lvl))
}

fn tracing(cfg: &Configuration) -> Logger {
    if cfg.tracing().log() {
        Logger::builder().build("tracing", LevelFilter::Info)
    } else {
        Logger::builder().build("tracing", LevelFilter::Off)
    }
}
