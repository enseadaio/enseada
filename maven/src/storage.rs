use maven_version::Version;

pub fn versioned_file_key(prefix: &str, version: &Version, filename: &str) -> String {
    format!("artifacts/maven/{}/{}/{}", prefix, version, filename)
}

pub fn file_key(prefix: &str, filename: &str) -> String {
    format!("artifacts/maven/{}/{}", prefix, filename)
}
