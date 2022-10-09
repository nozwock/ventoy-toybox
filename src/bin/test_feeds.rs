use anyhow::{Result, Ok};
use ventoy_toybox::core::utils::Feeds;


fn main() -> Result<()> {
    Feeds::new()?;
    Ok(())
}