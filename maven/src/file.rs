use maven_version::Version;

pub struct File<'a> {
    filename: &'a str,
    version: &'a Version,
    content: Vec<u8>,
}

impl<'a> File<'a> {
    pub fn new(filename: &'a str, version: &'a Version, content: Vec<u8>) -> Self {
        Self {
            filename,
            version,
            content,
        }
    }

    pub fn filename(&self) -> &str {
        self.filename
    }

    pub fn version(&self) -> &Version {
        self.version
    }

    pub fn content(&self) -> &[u8] {
        &self.content
    }

    pub fn into_content(self) -> Vec<u8> {
        self.content
    }
}
