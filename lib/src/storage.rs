pub use hold::*;

pub type Provider = Box<dyn hold::provider::Provider + Send + Sync>;
