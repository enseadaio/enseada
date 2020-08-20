use std::process::Command;

fn main() -> std::io::Result<()> {
    let current_dir = std::env::current_dir()?;
    let path = current_dir.join("docs/openapi.yml");
    let path = path.to_str().expect("openapi path");
    println!("cargo:rerun-if-changed={}", path);
    Command::new("yarn").arg("docs:build").status()?;
    #[cfg(debug_assertions)]
    println!(
        "cargo:rerun-if-changed={}",
        current_dir.join(".env").to_str().unwrap()
    );
    Ok(())
}
