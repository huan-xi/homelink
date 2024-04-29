use std::str::FromStr;
use anyhow::anyhow;
use sea_orm::{JsonValue, NotSet};
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use hap::characteristic::{Format, Perm, Unit};
use target_hap::hap_type_wrapper::HapTypeWrapper;
use target_hap::types::HapCharInfo;
use crate::db::entity::hap_characteristic::HapCharInfoQueryResult;
use crate::db::entity::prelude::{HapCharacteristicActiveModel, HapCharacteristicModel};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HapCharacteristicTemplate {
    pub cid: Option<i64>,
    pub service_id: Option<i64>,
    pub disabled: Option<bool>,
    pub char_type: HapTypeWrapper,
    #[serde(default)]
    pub info: HapCharInfoTemp,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub memo: Option<String>,
    /// 单位转换
    pub convertor: Option<String>,
    pub convertor_param: Option<JsonValue>,
}

impl HapCharacteristicTemplate {
    pub fn try_into_update_model(self) -> anyhow::Result<HapCharacteristicActiveModel> {
        Ok(HapCharacteristicActiveModel {
            cid: Set(self.cid.ok_or(anyhow!("cid不能为空"))?),
            service_id: self.service_id.map(|i| Set(i)).unwrap_or(NotSet),
            disabled: self.disabled.map(|i| Set(i)).unwrap_or(NotSet),
            name: self.name.clone().map(|i| Set(Some(i))).unwrap_or(NotSet),
            characteristic_type: Set(self.char_type.to_string()),
            convertor: self.convertor.clone().map(|i| Set(Some(i))).unwrap_or(NotSet),
            convertor_param: self.convertor_param.clone().map(|i| Set(Some(i))).unwrap_or(NotSet),
            memo: self.memo.clone().map(|i| Set(Some(i))).unwrap_or(NotSet),
            info: Set(HapCharInfoQueryResult(self.info.try_into()?)),
        })
    }

}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, Default)]
pub struct HapCharInfoTemp {
    pub format: Option<Format>,
    pub unit: Option<Unit>,
    pub min_value: Option<JsonValue>,
    pub max_value: Option<JsonValue>,
    pub step_value: Option<JsonValue>,
    pub max_len: Option<u16>,
    pub max_data_len: Option<u32>,
    pub valid_values: Option<Vec<JsonValue>>,
    pub valid_values_range: Option<Vec<JsonValue>>,
    pub ttl: Option<u64>,
    pub perms: Option<Vec<Perm>>,
    pub pid: Option<u64>,
}

impl TryInto<HapCharInfo> for HapCharInfoTemp {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<HapCharInfo, Self::Error> {
        Ok(HapCharInfo {
            format: self.format.ok_or(anyhow!("format is required"))?,
            unit: self.unit,
            min_value: self.min_value,
            max_value: self.max_value,
            step_value: self.step_value,
            max_len: self.max_len,
            max_data_len: self.max_data_len,
            valid_values: self.valid_values,
            valid_values_range: self.valid_values_range,
            ttl: self.ttl,
            perms: self.perms.ok_or(anyhow!("perms is required"))?,
            pid: self.pid,
        })
    }
}

impl From<HapCharInfo> for HapCharInfoTemp {
    fn from(value: HapCharInfo) -> Self {
        Self {
            format: Some(value.format),
            unit: value.unit,
            min_value: value.min_value,
            max_value: value.max_value,
            step_value: value.step_value,
            max_len: value.max_len,
            max_data_len: value.max_data_len,
            valid_values: value.valid_values,
            valid_values_range: value.valid_values_range,
            ttl: value.ttl,
            perms: Some(value.perms),
            pid: value.pid,
        }
    }
}


/// model 转模板
impl TryFrom<HapCharacteristicModel> for HapCharacteristicTemplate {
    type Error = anyhow::Error;


    fn try_from(value: HapCharacteristicModel) -> Result<Self, Self::Error> {
        Ok(
            Self {
                cid: Some(value.cid),
                service_id: Some(value.service_id),
                disabled: Some(value.disabled),
                char_type: HapTypeWrapper::from_str(value.characteristic_type.as_str())?,
                info: HapCharInfoTemp::from(value.info.0),
                name: value.name,
                memo: value.memo,
                convertor: value.convertor,
                convertor_param: value.convertor_param,
            }
        )
    }
}

