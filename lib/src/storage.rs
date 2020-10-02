use std::io;

use bytes::Bytes;
use futures::Stream;
pub use hold::*;
use std::pin::Pin;

pub type Provider = Box<dyn hold::provider::Provider + Send + Sync>;

pub type ByteChunk = std::result::Result<Bytes, io::Error>;
pub type ByteStream = Pin<Box<dyn Stream<Item = ByteChunk> + Send + 'static>>;
