use std::process::Command;

fn main() {
    yarte::recompile::when_changed();

    println!("cargo:rerun-if-changed=docs/openapi.yml");
    Command::new("yarn").arg("docs:build").status().unwrap();
}
