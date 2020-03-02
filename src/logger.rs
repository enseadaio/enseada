use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::Encode;
use log4rs::encode::json::JsonEncoder;
use log4rs::encode::pattern::PatternEncoder;
use log::{LevelFilter, Level};

use crate::config::CONFIG;
use std::str::FromStr;

fn encoder() -> Box<dyn Encode> {
    let fmt = CONFIG.log().format().to_lowercase();
    match fmt.as_str() {
        "json" => Box::new(JsonEncoder::new()),
        "text" | _ => Box::new(PatternEncoder::default()),
    }
}

fn level() -> LevelFilter {
    let lvl = CONFIG.log().level();
    Level::from_str(lvl.to_lowercase().as_str()).unwrap_or(Level::Info).to_level_filter()
}

pub fn init() {
    let stdout = ConsoleAppender::builder()
        .encoder(encoder())
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(level()))
        .unwrap();

    log4rs::init_config(config).unwrap();
}