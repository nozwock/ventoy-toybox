use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::{fs, io, path::PathBuf};

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

#[derive(Default, Debug, Clone)]
pub struct UpdateState {
    pub latest_release: Option<Release>,
    pub status: UpdateStatus,
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub enum UpdateStatus {
    #[default]
    Checking,
    CheckingDone,
    Downloading,
    Failed,
    Done,
}

/// Download a file from the internet
pub async fn download_file<T: ToString>(url: T, dest_file: PathBuf) -> Result<()> {
    let url = url.to_string();
    dbg!("downloading file from {}", &url);

    match ureq::get(&url).call() {
        Ok(resp) => {
            let mut file = fs::File::create(&dest_file).unwrap();

            if let Err(e) = io::copy(&mut resp.into_reader(), &mut file) {
                return Err(anyhow!("write failed!\n{}", e));
            }
        }
        Err(e) => return Err(anyhow!("req failed!\n{}", e)),
    }
    Ok(())
}
