use std::fmt::Debug;

use snafu::Snafu;

pub struct App;

#[derive(Debug, Snafu)]
pub enum AppError {
    #[snafu(display("{}", message))]
    ModuleInitializationFailed { message: String },
}

pub trait Module: Debug {
    fn boot(&self) -> Result<(), AppError>;
}
