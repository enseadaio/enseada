use std::str::FromStr;

use log::{Level, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::json::JsonEncoder;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::encode::Encode;

use crate::config::CONFIG;

fn encoder() -> Box<dyn Encode> {
    let fmt = CONFIG.log().format().to_lowercase();
    match fmt.as_str() {
        "json" => Box::new(JsonEncoder::new()),
        _ => Box::new(PatternEncoder::default()),
    }
}

fn level(lvl: &str) -> LevelFilter {
    Level::from_str(lvl.to_lowercase().as_str())
        .unwrap_or(Level::Info)
        .to_level_filter()
}

pub fn init() {
    let lvl = &CONFIG.log().level();
    let root_lvl = &CONFIG.log().root_level();

    let stdout = ConsoleAppender::builder().encoder(encoder()).build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .logger(Logger::builder().build("enseada_server", level(lvl)))
        .logger(Logger::builder().build("couchdb", level(lvl)))
        .build(Root::builder().appender("stdout").build(level(root_lvl)))
        .unwrap();

    log4rs::init_config(config).unwrap();
}
