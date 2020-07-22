pub const PATH_PREFIX: &str = "/static";

pub fn stylesheet_path() -> String {
    find_asset(|f| f.ends_with(".css")).expect("stylesheet path not found")
}

pub fn icon_path() -> String {
    find_asset(|f| f.starts_with("favicon")).expect("icon path not found")
}

pub fn logo_path() -> String {
    find_asset(|f| f.starts_with("enseada-logo")).expect("logo path not found")
}

fn find_asset<F: Fn(&str) -> bool>(fun: F) -> Option<String> {
    for file in crate::dashboard::Asset::iter() {
        if fun(&file) {
            return Some(format!("{}/{}", PATH_PREFIX, file));
        }
    }

    None
}
