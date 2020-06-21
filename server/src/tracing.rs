use crate::config::CONFIG;

pub fn init() {
    let cfg = CONFIG.tracing();

    let builder = tracing_subscriber::fmt().with_max_level(cfg.level());
    let res = if CONFIG.log().format().ne("json") {
        log::debug!("creating json tracer");
        builder.json().flatten_event(true).try_init()
    } else {
        log::debug!("creating regular tracer");
        builder.try_init()
    };
    if let Err(error) = res {
        panic!("{}", error)
    }
}
