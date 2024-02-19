use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use hap::characteristic::{Format, Perm, Unit};
use hap::HapType;
use hap_metadata::hap_metadata;
use hap_metadata::metadata::HapMetadata;
use crate::iot::characteristic_value::CharacteristicValue;
use crate::types::HapCharInfo;

pub fn get_default_type_info_map(meta: Arc<HapMetadata>) -> anyhow::Result<HashMap<HapType, HapCharInfo>> {
    let mut map = HashMap::new();


    let chars = &meta.characteristics;
    for (name, char) in chars {
        let str = char.short_uuid.as_str().trim_start_matches('0');
        match HapType::from_str(str) {
            Ok(hap_type) => {
                let format_str = format!("\"{}\"", char.format.as_str());
                let format: Format = serde_json::from_str(format_str.as_str())?;
                let unit = char
                    .units
                    .as_ref()
                    .map(|i| {
                        let unit = format!("\"{}\"", i.as_str());
                        let unit: Result<Unit, serde_json::Error> = serde_json::from_str(unit.as_str());
                        unit
                    })
                    .transpose()?;

                let min_value = char.min_value.as_ref().map(|i| {
                    CharacteristicValue::try_format(format, i.clone())
                        .map(|i| i.value)
                }).transpose()?;
                let max_value = char.max_value.as_ref().map(|i| {
                    CharacteristicValue::try_format(format, i.clone())
                        .map(|i| i.value)
                }).transpose()?;
                let step_value = char.step_value.as_ref().map(|i| {
                    CharacteristicValue::try_format(format, i.clone())
                        .map(|i| i.value)
                }).transpose()?;
                let max_len = char.max_length
                    .as_ref()
                    .and_then(|i| i.as_u64().map(|i| i as u16));
                let perms = hap_metadata::get_perms(char.properties as u64);
                let perm_str = serde_json::to_string(&perms)?;
                let perms: Vec<Perm> = serde_json::from_str(perm_str.as_str())?;
                let in_values = meta.characteristic_in_values.get(name.as_str());
                let out_values = meta.characteristic_out_values.get(name.as_str());
                let mut valid_values = None;
                if let Some(values) = in_values {
                    //枚举
                    valid_values = Some(values.clone().into_values().collect());
                };

                if let Some(values) = out_values {
                    //枚举
                    valid_values = Some(values.clone().into_values().collect());
                };


                let info = HapCharInfo {
                    format,
                    unit,
                    min_value,
                    max_value,
                    step_value,
                    max_len,
                    max_data_len: None,
                    valid_values,
                    valid_values_range: None,
                    ttl: None,
                    perms,
                    pid: None,
                };

                map.insert(hap_type, info);
            }
            Err(e) => {
                // let name = pascal_case(x.name.as_str());
                // println!("error,name:{name}:{:?}", x);
            }
        }
    }
    return Ok(map);
}





#[cfg(test)]
mod test {
    use std::sync::Arc;

    use hap_metadata::hap_metadata;
    use crate::hap_manager::default_char_info::get_default_type_info_map;


    #[test]
    pub fn test_default() -> anyhow::Result<()> {
        // SecuritySystemCurrentStateCharacteristic::new();

        let meta = hap_metadata().unwrap();
        // let a = HapManage::new();
        let a = get_default_type_info_map(Arc::new(meta))?;
        Ok(())
    }
}