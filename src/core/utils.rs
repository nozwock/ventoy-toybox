use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{anyhow, Result};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct FeedsItem {
    pub group: String,
    pub name: String,
    pub torrent_url: String,
    pub magnet: String,
    pub date: String,
}

// pub fn feeds_new() -> Result<Vec<FeedsItem>> {
//     let response = ureq::get(
//         "https://github.com/nozwock/ventoy-toybox-feed/releases/download/feeds/releases.json",
//     )
//     .call()?;
//     let feeds: Vec<FeedsItem> = response.into_json()?;
//     dbg!(&feeds);
//     Ok(feeds)
// }

pub fn find_file(path: &Path, file_name: &str) -> Result<PathBuf, String> {
    if path.is_dir() {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let entry_path = entry.path();
            dbg!(&entry_path);
            if entry_path.is_dir() {
                if let Ok(file) = find_file(entry_path.as_path(), file_name) {
                    return Ok(file);
                };
            } else if entry_path.ends_with(file_name) {
                return Ok(entry_path);
            }
        }
    }
    Err(format!("couldn't find {}", file_name))
}

pub fn open_in_explorer(path: &Path) -> anyhow::Result<()> {
    let cmd_name: &str;
    #[cfg(target_os = "windows")]
    {
        cmd_name = "explorer";
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        cmd_name = "xdg-open";
    }
    match dbg!(Command::new(cmd_name).arg(path.as_os_str()).spawn()) {
        Ok(_) => Ok(()),
        Err(err) => Err(anyhow!(
            "command failed: {} {}\n{}",
            cmd_name,
            path.to_str().unwrap(),
            err.to_string()
        )),
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn feeds_fetch() {
    //     let result = feeds_new();
    //     assert!(result.is_ok(), "fetch failed!\n{:?}", result);
    // }
}
