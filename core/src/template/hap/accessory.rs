use anyhow::anyhow;
use sea_orm::ActiveValue::Set;
use sea_orm::{JsonValue, NotSet};
use serde::{Deserialize, Serialize};
use target_hap::types::{CharIdentifier, ModelDelegateParam};
use crate::db::entity::hap_accessory::ModelDelegateParamVec;
use crate::db::entity::hap_bridge::BridgeCategory;
use crate::db::entity::prelude::{HapAccessoryActiveModel, HapAccessoryModel};
use crate::db::SNOWFLAKE;
use crate::init::manager::template_manager::ApplyMethod;
use crate::template::hap::service::ServiceTemplate;
use crate::template::hl_template::{default_str};


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModelDelegateParamTemplate {
    /// 为空则设置所有的chars
    pub chars: Option<Vec<CharIdentifier>>,
    ///配件模型 接管读写事件
    pub model: String,
    ///unit convert 设置,stag,ctag-> convertor
    /// 模型 运行时参数
    pub params: Option<JsonValue>,
}

impl TryInto<ModelDelegateParam> for ModelDelegateParamTemplate {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<ModelDelegateParam, Self::Error> {
        Ok(ModelDelegateParam {
            chars: self.chars.ok_or(anyhow!("delegate param chars is required"))?,
            model: self.model,
            params: self.params,
        })
    }
}

impl From<ModelDelegateParam> for ModelDelegateParamTemplate {
    fn from(value: ModelDelegateParam) -> Self {
        Self {
            chars: Some(value.chars),
            model: value.model,
            params: value.params,
        }
    }
}


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AccessoryTemplate {
    pub aid: Option<i64>,
    pub device_id: Option<i64>,
    pub bridge_id: Option<i64>,
    pub disabled: Option<bool>,
    /// 配件的类型
    pub category: BridgeCategory,
    #[serde(default = "default_str")]
    pub tag: String,
    pub memo: Option<String>,
    /// 配件的名称,默认取上一级
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub hap_delegates: Vec<ModelDelegateParamTemplate>,
    pub hap_delegate: Option<ModelDelegateParamTemplate>,
    pub services: Vec<ServiceTemplate>,
}

impl AccessoryTemplate {
    pub(crate) fn try_from_model(value: HapAccessoryModel, services: Vec<ServiceTemplate>) -> anyhow::Result<Self> {
        Ok(AccessoryTemplate {
            aid: Some(value.aid),
            device_id: Some(value.device_id),
            bridge_id: Some(value.bridge_id),
            disabled: Some(value.disabled),
            category: value.category,
            tag: value.tag.unwrap_or(default_str()),
            memo: value.memo,
            name: Some(value.name),
            hap_delegates: value.hap_model_delegates
                .0.into_iter()
                .map(|i| ModelDelegateParamTemplate::from(i))
                .collect(),
            hap_delegate: None,
            services,
        })
    }

    pub fn try_into_update_model(self) -> anyhow::Result<HapAccessoryActiveModel> {
        let now = Set(chrono::Local::now().naive_local());
        Ok(HapAccessoryActiveModel {
            aid: Set(self.aid.ok_or(anyhow!("aid不能为空"))?),
            name: Set(self.name.clone().ok_or(anyhow!("name不能为空"))?),
            tag: Set(Some(self.tag.clone())),
            device_id: Set(self.device_id.ok_or(anyhow!("device_id不能为空"))?),
            bridge_id: Set(self.bridge_id.ok_or(anyhow!("bridge_id不能为空"))?),
            disabled: Set(self.disabled.unwrap_or(false)),
            category: Set(self.category),
            hap_model_delegates: Set(ModelDelegateParamVec(self.hap_delegates
                .into_iter()
                .map(|i| i.try_into())
                .collect::<anyhow::Result<Vec<ModelDelegateParam>>>()?)),
            memo: Set(self.memo.clone()),
            info: Default::default(),
            temp_id: Default::default(),
            create_at: NotSet,
            update_at: now,
        })


        /* Ok(HapAccessoryActiveModel {
             aid: Set(match method {
                 ApplyMethod::Update => temp.aid.ok_or(anyhow!("aid不能为空"))?,
                 ApplyMethod::Create => SNOWFLAKE.next_id(),
             }),
             name: Set(temp.name.clone().ok_or(anyhow!("name不能为空"))?),
             tag: Set(Some(temp.tag.clone())),
             device_id: Set(temp.device_id.ok_or(anyhow!("device_id不能为空"))?),
             bridge_id: Set(temp.bridge_id.ok_or(anyhow!("bridge_id不能为空"))?),
             disabled: Set(temp.disabled.unwrap_or(false)),
             category: Set(temp.category),
             hap_model_delegates: Set(ModelDelegateParamVec(delegate)),
             memo: Set(temp.memo.clone()),
             info: Default::default(),
             temp_id: Default::default(),
             create_at: match method {
                 ApplyMethod::Create => now.clone(),
                 ApplyMethod::Update => NotSet
             },
             update_at: now,
         })*/
    }
    /*
        /// 模板转模型
        pub fn into_accessory_model(method: ApplyMethod, temp: &AccessoryTemplate) -> anyhow::Result<HapAccessoryActiveModel> {
            let now = Set(chrono::Local::now().naive_local());
            let mut delegate = vec![];
            for t in temp.hap_delegates.clone().into_iter() {
                delegate.push(t.try_into()?);
            }


        }*/
}


