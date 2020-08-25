use std::process::Command;

use walkdir::WalkDir;

fn main() -> std::io::Result<()> {
    for entry in WalkDir::new("./docs") {
        let entry = entry?;
        if entry.metadata()?.is_file() {
            if let Some(name) = entry.file_name().to_str() {
                println!("cargo:rerun-if-changed={}", name);
            }
        }
    }
    Command::new("yarn").arg("build").status()?;
    Ok(())
}
