use anyhow::anyhow;
use serde::Deserialize;
use std::{
    fs,
    io::{self, Write},
    path::Path,
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

pub fn write_resp_to_file<P>(resp: ehttp::Response, dest_file: P) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    if resp.ok {
        let mut file = fs::File::create(dest_file)?; // thread will panic if not file
        return Ok(file.write_all(&resp.bytes)?);
    }
    Err(anyhow!(resp.status_text))
}

#[cfg(unix)]
pub fn extract_targz<P>(archive_path: P, dest_dir: P) -> io::Result<()>
where
    P: AsRef<Path>,
{
    use flate2::read::GzDecoder;
    let mut archive = tar::Archive::new(GzDecoder::new(fs::File::open(archive_path)?));
    archive.unpack(dest_dir)?;
    Ok(())
}

#[cfg(windows)]
pub fn extract_zip<P>(archive_path: P, dest_dir: P) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let mut archive = zip::ZipArchive::new(fs::File::open(archive_path)?)?;
    archive.extract(dest_dir)?;
    Ok(())
}

pub const fn ventoy_bin_name() -> &'static str {
    #[cfg(windows)]
    {
        "Ventoy2Disk.exe"
    }
    #[cfg(target_os = "linux")]
    {
        "VentoyGUI.x86_64"
    }
}
