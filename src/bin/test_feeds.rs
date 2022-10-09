use anyhow::{Result, Ok};
use ventoy_toybox::feeds;


fn main() -> Result<()> {
    feeds::Feeds::new();
    Ok(())
}