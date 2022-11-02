use std::path::PathBuf;

pub const APP_NAME: &str = "Ventoy Toybox";

pub fn app_cache_dir() -> Option<PathBuf> {
    Some(dirs::cache_dir()?.join("ventoy-toybox"))
}
