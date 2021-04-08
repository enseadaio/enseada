use std::io::{Stderr, stderr};

use slog::{Drain, LevelFilter, Logger};

use crate::config::{Configuration, LogFormat};
use slog_async::Async;

pub fn create_logger(cfg: &Configuration) -> Logger {
    let drain = match cfg.log().format() {
        LogFormat::Text => text(),
        LogFormat::Json => json(),
    };
    let drain = LevelFilter::new(drain.fuse(), cfg.log().level()).fuse();
    Logger::root(drain, slog::o!())
}

fn text() -> Async {
    let decorator = slog_term::TermDecorator::new().stderr().build();
    let drain = slog_term::FullFormat::new(decorator).build();
    Async::new(drain.fuse()).build()
}

fn json() -> Async {
    let drain = slog_json::Json::default(stderr());
    Async::new(drain.fuse()).build()
}

