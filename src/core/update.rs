use anyhow::anyhow;
use serde::Deserialize;
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize, Clone)]
pub struct Release {
    pub tag_name: String,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ReleaseAsset {
    pub name: String,
    #[serde(rename = "browser_download_url")]
    pub download_url: String,
}

/// Download a file from the internet
pub async fn download_file<T: ToString>(url: T, dest_file: &PathBuf) -> anyhow::Result<()> {
    let url = url.to_string();
    dbg!("downloading file from {}", &url);

    match ureq::get(&url).call() {
        Ok(resp) => {
            let mut file = fs::File::create(dest_file).unwrap();

            if let Err(e) = io::copy(&mut resp.into_reader(), &mut file) {
                return Err(anyhow!("write failed!\n{}", e));
            }
        }
        Err(e) => return Err(anyhow!("req failed!\n{}", e)),
    }
    Ok(())
}

pub fn write_resp_to_file(resp: ehttp::Response, dest_file: PathBuf) -> Result<PathBuf, String> {
    if resp.ok {
        let mut file = fs::File::create(&dest_file).unwrap(); // thread will panic if not file
        dbg!(&file);

        match file.write_all(&resp.bytes) {
            Ok(_) => return Ok(dest_file),
            Err(err) => return Err(err.to_string()),
        };
    }
    Err(resp.status_text)
}

pub const fn ventoy_bin_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "Ventoy2Disk.exe"
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        "VentoyGUI.x86_64"
    }
}
