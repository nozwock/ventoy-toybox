use std::path::PathBuf;

pub const APP_NAME: &str = "Ventoy Toybox";
pub const APP_DIR: &str = "com.github.nozwock.ventoy-toybox";

#[cfg(feature = "app-icon")]
pub const APP_ICON: &[u8] = include_bytes!("../assets/ferris64.png");

pub fn app_cache_dir() -> Option<PathBuf> {
    Some(dirs::cache_dir()?.join(APP_DIR))
}

pub fn app_cache_path() -> PathBuf {
    app_cache_dir().unwrap_or_default().join("cache.ron")
}
