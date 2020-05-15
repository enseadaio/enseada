use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=./docs/openapi.yml");
    Command::new("yarn").arg("docs:build").status().unwrap();
}
