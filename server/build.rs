use std::process::Command;

fn main() -> std::io::Result<()> {
    let current_dir = std::env::current_dir()?;
    #[cfg(debug_assertions)]
    println!(
        "cargo:rerun-if-changed={}",
        current_dir.join(".env").to_str().unwrap()
    );

    let dashboard_dir = current_dir.join("..").join("dashboard");
    println!(
        "cargo:rerun-if-changed={}",
        dashboard_dir.join("package.json").to_str().unwrap()
    );
    Command::new("yarn")
        .current_dir(dashboard_dir.as_path())
        .arg("install")
        .status()?;
    Command::new("yarn")
        .current_dir(dashboard_dir.as_path())
        .arg("build")
        .status()?;
    Ok(())
}
