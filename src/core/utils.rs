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

pub fn find_file<P>(path: P, file_name: &str) -> Result<PathBuf, String>
where
    P: AsRef<Path>,
{
    if path.as_ref().is_dir() {
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

pub fn open_in_explorer<P>(path: P) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    let cmd_name: &str;
    #[cfg(windows)]
    {
        cmd_name = "explorer";
    }
    #[cfg(target_os = "linux")]
    {
        cmd_name = "xdg-open";
    }
    match dbg!(Command::new(cmd_name)
        .arg(path.as_ref().as_os_str())
        .spawn())
    {
        Ok(_) => Ok(()),
        Err(err) => Err(anyhow!(
            "command failed: {} {}\n{}",
            cmd_name,
            path.as_ref().to_str().unwrap(),
            err.to_string()
        )),
    }
}
