use walkdir::WalkDir;
use std::ffi::OsStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    for entry in WalkDir::new("./proto") {
        let entry = entry?;
        if entry.metadata()?.is_file() && entry.path().extension() == Some(OsStr::new("proto")){
            if let Some(path) = entry.path().to_str() {
                println!("cargo:rerun-if-changed={}", path);
                files.push(path.to_string());
            }
        }
    }
    eprintln!("{:?}", files);
    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .type_attribute(".", "#[derive(::serde::Deserialize, ::serde::Serialize)]")
        .compile(&files,&["./proto".to_string()])?;
    Ok(())
}
