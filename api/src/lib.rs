pub mod error;
pub mod meta;
pub mod tls;
pub mod users;
pub mod watch;
mod client;

pub use client::Client;
pub use tonic;

pub type Result<T> = std::result::Result<T, tonic::Status>;
