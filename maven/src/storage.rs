use maven_version::Version;

pub fn file_key(prefix: &str, version: &Version, filename: &str) -> String {
    format!("artifacts/maven/{}/{}/{}", prefix, version, filename)
}
