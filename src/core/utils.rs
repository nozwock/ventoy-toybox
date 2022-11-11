use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::anyhow;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct FeedsItem {
    pub group: String,
    pub name: String,
    pub torrent_url: String,
    pub magnet: String,
    pub date: String,
}

pub fn find_file<P>(path: P, fname: &str) -> io::Result<PathBuf>
where
    P: AsRef<Path>,
{
    if path.as_ref().is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?.path();
            if entry.is_dir() {
                if let Ok(file) = find_file(entry.as_path(), fname) {
                    return Ok(file);
                };
            } else if entry.is_file() && entry.ends_with(fname) {
                return Ok(entry);
            }
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("couldn't find {}", fname),
    ))
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
    match Command::new(cmd_name)
        .arg(path.as_ref().as_os_str())
        .spawn()
    {
        Ok(_) => Ok(()),
        Err(err) => Err(anyhow!(
            "command failed: {} {}\n{}",
            cmd_name,
            path.as_ref()
                .to_str()
                .ok_or_else(|| anyhow!("path is not valid unicode"))?,
            err.to_string()
        )),
    }
}

#[cfg(windows)]
pub fn string_to_pcstr(string: &str) -> windows::core::PCSTR {
    windows::core::PCSTR::from_raw(format!("{string}{}", '\0').as_str().as_ptr())
}

#[cfg(windows)]
pub fn runas_admin<P>(path: P) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    use windows::{
        s, Win32::Foundation::GetLastError, Win32::UI::Shell::ShellExecuteA,
        Win32::UI::WindowsAndMessaging::SW_NORMAL,
    };

    let fpath = string_to_pcstr(
        path.as_ref()
            .to_str()
            .ok_or_else(|| anyhow!("failed to convert path to string"))?,
    );

    let pwd = string_to_pcstr(
        path.as_ref()
            .parent()
            .ok_or_else(|| anyhow!("root can't have a parent"))?
            .to_str()
            .ok_or_else(|| anyhow!("failed to convert path to string"))?,
    );

    let result;
    unsafe {
        result = ShellExecuteA(None, s!("runas"), fpath, s!(""), pwd, SW_NORMAL);
        println!("{}", GetLastError().0);
    }

    match result.0 {
        x if x > 32 => Ok(()),
        _ => Err(anyhow!("Admin privileges denied!")),
    }
}
