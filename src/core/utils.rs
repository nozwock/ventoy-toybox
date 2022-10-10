use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Feeds;

#[derive(Deserialize, Debug)]
pub struct FeedsItem {
    pub group: String,
    pub name: String,
    pub torrent_url: String,
    pub magnet: String,
    pub date: String,
}

impl Feeds {
    pub fn new() -> Result<Vec<FeedsItem>> {
        let response = ureq::get(
            "https://github.com/nozwock/ventoy-toybox-feed/releases/download/feeds/releases.json",
        )
        .call()?;
        let feeds: Vec<FeedsItem> = response.into_json()?;
        dbg!(&feeds);
        return Ok(feeds);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feeds_fetch() {
        let result = Feeds::new();
        assert!(result.is_ok(), "fetch failed!\n{:?}", result);
    }
}
