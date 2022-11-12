use std::path::PathBuf;

pub const APP_NAME: &str = "Ventoy Toybox";

#[cfg(feature = "app-icon")]
pub const APP_ICON: &[u8] = include_bytes!("../assets/ferris64.png");

pub fn app_cache_dir() -> Option<PathBuf> {
    Some(dirs::cache_dir()?.join("ventoy-toybox"))
}
