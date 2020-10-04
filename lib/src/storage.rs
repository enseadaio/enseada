use std::io;
use std::pin::Pin;

pub use bytes::*;
use futures::Stream;
pub use hold::*;

pub type Provider = Box<dyn hold::provider::Provider + Send + Sync>;

pub type ByteChunk = std::result::Result<Bytes, io::Error>;
pub type ByteStream = Pin<Box<dyn Stream<Item = ByteChunk> + Send + Sync + 'static>>;
