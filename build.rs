use std::{fs, env};
use std::path::Path;

fn main() {
    yarte::recompile::when_changed();

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("openapi.yml");
    fs::copy("./docs/openapi.yml", &out_path).unwrap();
}
