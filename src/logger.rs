use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::Encode;
use log4rs::encode::json::JsonEncoder;
use log4rs::encode::pattern::PatternEncoder;
use log::{LevelFilter, Level};

use crate::config::Configuration;
use std::str::FromStr;

fn encoder(cfg: &Configuration) -> Box<dyn Encode> {
    let fmt = cfg.log().format().clone();
    match fmt.map(|f| f.to_lowercase()).as_deref() {
        Some("json") => Box::new(JsonEncoder::new()),
        Some(_) | None => Box::new(PatternEncoder::default()),
    }
}

fn level(cfg: &Configuration) -> LevelFilter {
    let lvl = cfg.log().level().clone();
    Level::from_str(lvl.to_lowercase().as_str()).unwrap_or(Level::Info).to_level_filter()
}

pub fn init(cfg: &Configuration) {
    let stdout = ConsoleAppender::builder()
        .encoder(encoder(cfg))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(level(cfg)))
        .unwrap();

    log4rs::init_config(config).unwrap();
}