use std::process::Command;

fn main() {
    yarte::recompile::when_changed();

    Command::new("yarn").arg("docs:build").status().unwrap();
}
