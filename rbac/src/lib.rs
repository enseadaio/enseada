pub use enforcer::{Enforcer, EvaluationError};
pub use rule::Rule;
pub use watcher::Watcher;

mod enforcer;
mod model;
pub mod role;
mod rule;
mod watcher;

static ROOT_USER: &str = "user:root";
