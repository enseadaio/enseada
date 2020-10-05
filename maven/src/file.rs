use futures::{stream, Stream};

use enseada::storage::{ByteChunk, ByteStream, Bytes};
use maven_version::Version;

pub struct File<'a> {
    filename: &'a str,
    version: Option<&'a Version>,
    size: usize,
    content: ByteStream,
}

impl<'a> File<'a> {
    pub fn new<S: Stream<Item = ByteChunk> + Send + Sync + 'static>(
        version: Option<&'a Version>,
        filename: &'a str,
        size: usize,
        content: S,
    ) -> Self {
        Self {
            filename,
            version,
            size,
            content: Box::pin(content),
        }
    }

    pub fn filename(&self) -> &str {
        self.filename
    }

    pub fn version(&self) -> Option<&Version> {
        self.version
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn into_byte_stream(self) -> ByteStream {
        self.content
    }

    pub fn from_bytes(version: Option<&'a Version>, filename: &'a str, bytes: Bytes) -> Self {
        Self::new(
            version,
            filename,
            bytes.len(),
            stream::once(async move { Ok(bytes) }),
        )
    }
}

pub struct FilePointer {
    filename: String,
    version: Option<Version>,
    prefix: String,
}

impl FilePointer {
    pub fn filename(&self) -> &str {
        &self.filename
    }
    pub fn version(&self) -> Option<&Version> {
        self.version.as_ref()
    }
    pub fn prefix(&self) -> &str {
        &self.prefix
    }
}

pub fn parse_file_path(path: &str) -> Option<FilePointer> {
    let mut reversed = path.trim_start_matches('/').split('/').rev();
    let filename = reversed.nth(0);
    if filename.is_none() {
        return None;
    }
    let filename = filename.unwrap().to_string();

    let maybe_version = reversed.nth(0);
    if maybe_version.is_none() {
        return None;
    }
    let maybe_version = maybe_version.unwrap().to_string();
    let mut rest: Vec<String> = reversed.rev().map(str::to_string).collect();
    let version = Version::parse(&maybe_version).ok();
    if let Some(version) = version {
        Some(FilePointer {
            filename,
            version: Some(version),
            prefix: rest.join("/"),
        })
    } else {
        rest.push(maybe_version);
        Some(FilePointer {
            filename,
            version: None,
            prefix: rest.join("/"),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_parses_a_file_pointer_with_version() {
        let path = "/io/enseada/test/1.0-alpha/test";
        let exp_version = Version::parse("1.0-alpha").unwrap();

        let pointer = parse_file_path(path).unwrap();

        assert_eq!("test", pointer.filename());
        assert_eq!(Some(&exp_version), pointer.version());
        assert_eq!("io/enseada/test", pointer.prefix());
    }

    #[test]
    fn it_parses_a_file_pointer_without_version() {
        let path = "io/enseada/test/test";
        let pointer = parse_file_path(path).unwrap();

        assert_eq!("test", pointer.filename());
        assert_eq!(None, pointer.version());
        assert_eq!("io/enseada/test", pointer.prefix());
    }

    #[test]
    fn it_does_not_parse_an_empty_file_pointer() {
        let path = "";
        let pointer = parse_file_path(path);

        assert!(pointer.is_none());
    }

    #[test]
    fn it_does_not_parse_a_single_slash() {
        let path = "/";
        let pointer = parse_file_path(path);

        assert!(pointer.is_none());
    }

    #[test]
    fn it_does_not_parse_an_short_file_pointer() {
        let path = "/test";
        let pointer = parse_file_path(path);

        assert!(pointer.is_none());
    }
}
