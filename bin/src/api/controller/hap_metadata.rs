use std::collections::HashMap;
use axum::extract::{Path, State};
/// hap 元数据
use hap_metadata::{convert_to_kebab_case, hap_metadata};
use hap_metadata::metadata::HapService;
use crate::api::output::{ApiResp, ApiResult, err_msg, ok_data};
use crate::api::results::{CharacteristicMetaResult, ServiceMetaResult};
use crate::api::state::AppState;


// 获取所有特征类型

pub async fn get_characteristic_meta(state: State<AppState>, Path(hap_type): Path<String>) -> ApiResult<CharacteristicMetaResult> {
    let mut characteristics = HashMap::new();
    state.hap_metadata.characteristics
        .values()
        .for_each(|c| {
            //名称
            let key = hap_metadata::utils::pascal_case(c.name.as_str());
            characteristics.insert(key, c.clone());
        });
    match characteristics.get(hap_type.as_str()) {
        None => {
            err_msg("characteristic not found")
        }
        Some(c) => {
            ok_data(CharacteristicMetaResult::from_ch(c, ""))
        }
    }
}


pub async fn get_service_meta(state: State<AppState>, Path(hap_type): Path<String>) -> ApiResult<ServiceMetaResult> {
    match state.hap_metadata.services.get(hap_type.as_str()) {
        None => {
            err_msg("service not found")
        }
        Some(svc) => {
            let mut svc = svc.clone();
            let req = svc.characteristics.required_characteristics;
            let required = req.iter()
                .map(|c| {
                    let ch = state.hap_metadata.characteristics.get(c).unwrap();
                    CharacteristicMetaResult::from_ch(ch, c.as_str())
                }).collect();
            let optional = svc.characteristics.optional_characteristics
                .unwrap_or(vec![])
                .iter()
                .map(|c| {
                    let ch = state.hap_metadata.characteristics.get(c).unwrap();
                    CharacteristicMetaResult::from_ch(ch, c.as_str())
                }).collect();
            ok_data(ServiceMetaResult { required, optional })
        }
    }
}


#[test]
pub fn test() -> anyhow::Result<()> {
    let metadata = hap_metadata()?;
    let s1 = "CurrentTemperature";
    // let s2 = convert_to_kebab_case(s1);
    // println!("{}", s2); // 输出 color-temperature
    let a = metadata.characteristics
        .get(s1).unwrap();
    // let mut a = a.clone();
    // a.name = a.name.replace(" ", "").to_string();
    let str = serde_json::to_string(&a)?;
    println!("{}", str);

    println!("{:#?}", metadata);

    Ok(())
}

#[test]
pub fn test_service() -> anyhow::Result<()> {
    let metadata = hap_metadata()?;
    let s1 = "Lightbulb";
    // let s2 = convert_to_kebab_case(s1);
    // println!("{}", s2); // 输出 color-temperature
    let a = metadata.services.get(s1).unwrap();
    let mut svc = a.clone();

    let req = svc.characteristics.required_characteristics;
    // let map = HashMap::new();
    /* metadata.characteristics.values()
         .for_each(|v|{
             v.name
         })
 */
    let required = req.iter().map(|c| {
        let ch = metadata.characteristics.get(c).unwrap();
        CharacteristicMetaResult::from_ch(ch, c.as_str())
        //svc.characteristics.required_characteristics_meta.push(c.clone());
    }).collect();
    let res = ServiceMetaResult { required, optional: vec![] };
    // a.name = a.name.replace(" ", "").to_string();
    let str = serde_json::to_string(&a)?;
    println!("{}", str);


    Ok(())
}