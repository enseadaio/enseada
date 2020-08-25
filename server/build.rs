fn main() -> std::io::Result<()> {
    let current_dir = std::env::current_dir()?;
    #[cfg(debug_assertions)]
    println!(
        "cargo:rerun-if-changed={}",
        current_dir.join(".env").to_str().unwrap()
    );
    Ok(())
}
