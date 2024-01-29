use anyhow::anyhow;
use rust_embed::RustEmbed;

use crate::metadata::{HapMetadata, SystemMetadata};

pub mod metadata;
pub mod utils;

#[derive(RustEmbed)]
#[folder = "gen"]
struct Asset;



pub fn hap_metadata() -> anyhow::Result<HapMetadata> {
    let file = Asset::get("system.json")
        .ok_or(anyhow::anyhow!("system.json not found"))?;
    let bytes = file.data.as_ref();
    let metadata: SystemMetadata = serde_json::from_slice(bytes)
        .map_err(|_e| anyhow!("json 格式不正确"))?;
    Ok(HapMetadata::from(metadata))
}

/// ColorTemperature 转 color-temperature
pub fn convert_to_kebab_case(s: &str) -> String {
    let mut result = String::new();
    let mut is_first_char = true;
    for c in s.chars() {
        if c.is_ascii_uppercase() {
            if !is_first_char {
                result.push('-');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
        is_first_char = false;
    }
    result
}


#[test]
pub fn test() {
    let metadata = hap_metadata();
    println!("{:#?}", metadata);
}