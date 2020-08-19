pub use enforcer::{Enforcer, EvaluationError};
pub use rule::Rule;
pub use watcher::Watcher;

mod enforcer;
mod model;
mod rule;
mod watcher;

static ROOT_USER: &str = "user:root";
