use anyhow::{Ok, Result};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Feeds {}

#[derive(Deserialize, Debug)]
struct FeedsItem {
    group: String,
    name: String,
    torrent_url: String,
    magnet: String,
    date: String,
}

impl Feeds {
    pub fn new() -> Result<()> {
        const URL: &str =
            "https://github.com/nozwock/ventoy-toybox-feed/releases/download/feeds/releases.json";
        let response = ureq::get(URL).call()?;
        let feeds = response.into_string()?;
        let feeds: Vec<FeedsItem> = serde_json::from_str(&feeds)?;
        dbg!(&feeds);
        return Ok(());
    }
}
