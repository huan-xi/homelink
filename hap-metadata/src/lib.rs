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


/// 获取权限vec
pub fn get_perms(value: u64) -> Vec<String> {
    let perms_map = vec![
        (1 << 0, "ev".to_string()),
        (1 << 1, "pr".to_string()),
        (1 << 2, "pw".to_string()),
        // Relevant for Bluetooth.
        // (1 << 3, "\n\t\t\t\tPerm::Broadcast,".to_string()),
        // aa set by homed just signals that aa may be supported. Setting up aa will always require a custom made app
        // though. (1 << 4, "\n\t\t\t\tPerm::AdditionalAuthorization,".to_string()),
        (1 << 5, "tw".to_string()),
        (1 << 6, "hd".to_string()),
        (1 << 7, "wr".to_string()),
    ];
    let properties_bitmap = value;
    let mut perms = vec![];
    for (bitmap, name) in perms_map {
        // if it stays the same, the bit is set
        if (bitmap | properties_bitmap) == properties_bitmap {
            perms.push(name.clone());
        }
    }
    perms
}


#[cfg(test)]
mod test {
    use crate::hap_metadata;

    #[test]
    pub fn test() {
        let characteristics = hap_metadata().unwrap().characteristics;
        let mut characteristic = None;
        characteristics
            .values()
            .for_each(|c| {
                //名称
                let key = crate::utils::pascal_case(c.name.as_str());
                if key.as_str() == "SecuritySystemCurrentState" {
                    characteristic = Some(c.clone());
                };
                // characteristics.insert(key, c.clone());
            });
        println!("{:?}", characteristic);
        if let Some(c) = characteristic {
            //解析prop
            let perms = crate::get_perms(c.properties as u64);
            println!("{:?}", perms);
        };
    }
}

