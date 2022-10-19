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
    anyhow::Ok(())
}

pub fn write_resp_to_file(resp: ehttp::Response, dest_file: &PathBuf) -> Result<(), String> {
    if resp.ok {
        let mut file = fs::File::create(dest_file).unwrap(); // thread will panic if not file
        dbg!(&file);

        match file.write_all(&resp.bytes) {
            Ok(_) => return Ok(()),
            Err(err) => return Err(err.to_string()),
        };
    }
    Err(resp.status_text)
}

#[cfg(not(target_os = "windows"))]
pub fn extract_targz(archive_path: &Path, dest_dir: &Path) {
    use flate2::read::GzDecoder;
    use fs::File;
    let mut archive = tar::Archive::new(GzDecoder::new(File::open(archive_path).unwrap()));
    archive.unpack(dest_dir).unwrap();
}

#[cfg(target_os = "windows")]
pub fn extract_zip(archive_path: &Path, dest_dir: &Path) {
    use fs::File;
    let mut archive = zip::ZipArchive::new(File::open(archive_path).unwrap()).unwrap();
    archive.extract(dest_dir).unwrap();
}

pub fn find_file(path: &Path, file_name: &str) -> Result<PathBuf, String> {
    if path.is_dir() {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let entry_path = entry.path();
            dbg!(&entry_path);
            if entry_path.is_dir() {
                match find_file(&entry_path.as_path(), file_name) {
                    Ok(file) => return Ok(file),
                    Err(_) => (),
                };
            } else if entry_path.ends_with(file_name) {
                return Ok(entry_path);
            }
        }
    }
    Err(format!("couldn't find {}", file_name))
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
