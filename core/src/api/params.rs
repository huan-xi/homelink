pub mod power;
pub mod template;

use std::str::FromStr;
use sea_orm::ActiveValue::Set;
use sea_orm::JsonValue;
use serde::Serialize;
use crate::hap::hap_type::MappingHapType;
use serde_aux::prelude::deserialize_number_from_string;
use crate::db::entity::hap_characteristic::HapCharInfoQueryResult;
use serde_aux::prelude::deserialize_option_number_from_string;
use target_hap::hap_type_wrapper::HapTypeWrapper;
use target_hap::types::HapCharInfo;
use crate::db::entity::common::PropertyVec;
use crate::db::entity::hap_bridge::BridgeCategory;
use crate::db::entity::iot_device::SourcePlatform;
use crate::db::entity::prelude::{HapAccessoryActiveModel, HapCharacteristicActiveModel};
use crate::init::manager::template_manager::BridgeMode;
use crate::template::hl_template::TemplateFormat;

#[derive(serde::Deserialize, Debug)]
pub struct LoginParam {
    /// 账号
    pub password: String,
    pub username: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct AddHapBridgeParam {
    /// pin码
    pub pin_code: Option<String>,
    pub category: BridgeCategory,
    pub name: String,

}

#[derive(serde::Deserialize, Debug)]
pub struct AccountParam {
    /// 账号
    pub account: String,
    pub password: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct DidParam {
    /// did
    pub did: String,
}


#[derive(serde::Deserialize, Debug)]
pub struct MiConvertByTemplateParam {
    /// 使用的模板id
    pub id: String,
    pub did: String,
    /// 设备列表
    // pub devices: Vec<String>,
    pub bridge_mode: BridgeMode,
    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    pub bridge_id: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    pub gateway_id: Option<i64>,
}

#[derive(serde::Deserialize, Debug)]
pub struct MiConvertToIotParam {
    /// did
    pub did: String,
    pub name: String,
    pub integration: String,
    pub memo: Option<String>,
}


#[derive(serde::Deserialize, Debug)]
pub struct QueryIotDeviceParam {
    pub device_type: Option<Vec<String>>,
    pub is_gateway: Option<bool>,
    pub source_platform: Option<SourcePlatform>,
}

#[derive(serde::Deserialize, Debug)]
pub struct EditDeviceParam {
    pub device_type: Option<String>,
    #[serde(deserialize_with = "deserialize_option_number_from_string")]
    pub gateway_id: Option<i64>,
    pub name: Option<String>,
    pub is_gateway: Option<bool>,
    pub memo: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct TestPropParam {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub siid: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub piid: i32,
    pub value: Option<JsonValue>,
}



#[derive(serde::Deserialize, Debug)]
pub struct GetTemplateParam {
    pub format: TemplateFormat,
}

#[derive(serde::Deserialize, Debug)]
pub struct AddServiceParam {
    pub memo: Option<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub accessory_id: i64,
    pub configured_name: Option<String>,
    /// 服务类型
    pub service_type: HapTypeWrapper,
    pub characteristics: Vec<CharacteristicParam>,
}




#[derive(serde::Deserialize, Debug)]
pub struct CharacteristicParam {
    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    pub cid: Option<i64>,
    pub characteristic_type: String,
    pub info: HapCharInfo,
    pub name: Option<String>,
    pub convertor: Option<String>,
}

impl CharacteristicParam {
    pub fn into_model(self, service_id: i64) -> anyhow::Result<HapCharacteristicActiveModel> {
        // let format: Format = serde_json::from_str(format!("\"{}\"", self.format.as_str()).as_str()).map_err(|e| anyhow!("格式转换错误:{:?}", e))?;
        let model = HapCharacteristicActiveModel {
            cid: Default::default(),
            characteristic_type: Set(self.characteristic_type.clone()),
            service_id: Set(service_id),
            disabled: Set(false),
            convertor: Set(self.convertor),
            info: Set(HapCharInfoQueryResult(self.info)),
            ..Default::default()
        };


        Ok(model)
    }
}




#[derive(serde::Deserialize, Debug)]
pub struct DisableParam {
    pub disabled: bool,
}




#[derive(serde::Deserialize, Debug)]
pub struct Test {
    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    pub cid: Option<i32>,
}


#[test]
pub fn test() {
    let str = r#"{"characteristic_type":"PowerState","mapping_method":"MIotSpec","service_id":"1194242687084003328","mapping_property":{"siid":"2","piid":"1"},"format":"bool","name":"on"}"#;
    let param: Test = serde_json::from_str(str).unwrap();
    println!("{:?}", param);
}

#[test]
pub fn test2() {
    let a = MappingHapType::Name;
    let str = serde_json::to_string(&a).unwrap();
    println!("{}", str);
    println!("{:?}", a);
}