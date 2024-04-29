use std::str::FromStr;
use anyhow::anyhow;
use sea_orm::ActiveValue::Set;
use sea_orm::NotSet;
use serde::{Deserialize, Serialize};
use target_hap::hap_type_wrapper::HapTypeWrapper;
use crate::db::entity::prelude::{HapCharacteristicModel, HapServiceActiveModel, HapServiceModel};
use crate::init::manager::template_manager::ApplyMethod;
use crate::template::hap::chars::HapCharacteristicTemplate;
use crate::template::hl_template::default_str;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceTemplate {
    pub service_id: Option<i64>,
    pub accessory_id: Option<i64>,
    /// 配件的类型
    pub service_type: HapTypeWrapper,
    pub chars: Vec<HapCharacteristicTemplate>,
    #[serde(default = "default_str")]
    pub tag: String,
    pub configured_name: Option<String>,
    pub memo: Option<String>,
    pub primary: Option<bool>,
    pub disabled: Option<bool>,
}

impl ServiceTemplate {
    pub(crate) fn try_from_model(value: HapServiceModel, chars: Vec<HapCharacteristicModel>) -> anyhow::Result<Self> {
        Ok(
            Self {
                service_id: Some(value.id),
                accessory_id: Some(value.accessory_id),
                service_type: HapTypeWrapper::from_str(value.service_type.as_str())?,
                chars: chars.into_iter()
                    .map(|i| HapCharacteristicTemplate::try_from(i))
                    .collect::<Result<Vec<HapCharacteristicTemplate>, anyhow::Error>>()?,
                tag: value.tag.unwrap_or(default_str()),
                configured_name: value.configured_name,
                memo: value.memo,
                primary: Some(value.primary),
                disabled: Some(value.disabled),
            }
        )
    }

    pub fn try_into_update_model(self) -> anyhow::Result<HapServiceActiveModel> {
        Ok(HapServiceActiveModel {
            id: Set(self.service_id.ok_or(anyhow!("service id不能为空"))?),
            tag: Set(Some(self.tag.clone())),
            accessory_id:self.accessory_id.map_or(NotSet, |s| Set(s)),
            configured_name: Set(self.configured_name.clone()),
            memo: Set(self.memo.clone()),
            service_type: Set(self.service_type.to_string()),
            disabled: self.disabled.map_or(NotSet, |s| Set(s)),
            primary: self.primary.map_or(NotSet, |s| Set(s)),
        })
    }
}
