use std::process::Command;

fn main() {
    let current_dir = std::env::current_dir().expect("env::current_dir");
    let path = current_dir.join("docs/openapi.yml");
    let path = path.to_str().expect("openapi path");
    println!("cargo:rerun-if-changed={}", path);
    Command::new("yarn").arg("docs:build").status().unwrap();
}
