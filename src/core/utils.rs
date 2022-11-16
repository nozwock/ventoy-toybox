use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FeedsItem {
    pub group: String,
    pub name: String,
    pub torrent_url: String,
    pub magnet: String,
    pub date: String,
}

pub fn find_file<P>(path: P, fname: &str) -> Option<PathBuf>
where
    P: AsRef<Path>,
{
    if path.as_ref().is_dir() {
        for entry in fs::read_dir(path).ok()? {
            let entry = entry.ok()?.path();
            if entry.is_dir() {
                if let Some(file) = find_file(entry.as_path(), fname) {
                    return Some(file);
                };
            } else if entry.is_file() && entry.ends_with(fname) {
                return Some(entry);
            }
        }
    }
    None
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
        _ => Err(anyhow!("admin privileges denied!")),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use fs::File;
    use tempdir::TempDir;

    #[test]
    fn file_found() {
        let tmpdir = TempDir::new("test").unwrap();
        let file_name = "some.file";
        File::create(tmpdir.path().join(file_name)).unwrap();
        find_file(tmpdir.path(), file_name).expect("failed to find the file");
    }

    #[test]
    #[should_panic(expected = "failed to find the file")]
    fn file_not_found() {
        let tmpdir = TempDir::new("test").unwrap();
        File::create(tmpdir.path().join("some.file")).unwrap();
        find_file(tmpdir.path(), "wrong.file").unwrap_or_else(|| panic!("failed to find the file"));
    }
}
