use std::collections::HashMap;
use std::sync::Arc;
use anyhow::anyhow;
use bimap::BiMap;
use impl_new::New;
use hap::characteristic::delegate::{CharReadParam, CharReadResult, CharUpdateParam, CharUpdateResult};
use hap::HapType;
use log::{info, warn};
use crate::hap::models::{AccessoryModelExt, AccessoryModelExtConstructor, AccessoryModelExtPointer, ContextPointer, ReadValueResult, UpdateValueResult};
use sea_orm::JsonValue;
use miot_spec::proto::miio_proto::MiotSpecId;
use crate::hap::hap_type::MappingHapType;

fn default_str() -> String {
    "default".to_string()
}

#[derive(Debug, serde::Deserialize)]
pub struct PropMappingParam {
    /// stag,ctag,{siid,piid}
    #[serde(default="default_str")]
    stag: String,
    ctag: MappingHapType,
    siid: i32,
    piid: i32,
}

#[derive(New, Eq, PartialEq, Hash, Debug)]
pub struct Cid {
    stag: Option<String>,
    ctag: MappingHapType,
}


/// 属性映射模型
pub struct ModelExt {
    ctx: ContextPointer,
    mapping: BiMap<Cid, MiotSpecId>,
}


impl AccessoryModelExtConstructor for ModelExt {
    fn new(ctx: ContextPointer, params: Option<JsonValue>) -> anyhow::Result<AccessoryModelExtPointer> {
        let params = params.ok_or(anyhow!("mode switch params is none"))?;
        let params: Vec<PropMappingParam> = serde_json::from_value(params)?;
        let mut mapping = BiMap::new();
        for param in params {
            mapping.insert(Cid::new(param.stag, param.ctag), MiotSpecId::new(param.siid, param.piid));
        }
        Ok(Arc::new(Self {
            ctx,
            mapping,
        }))
    }
}


#[async_trait::async_trait]
impl AccessoryModelExt for ModelExt {
    async fn read_chars_value(&self, params: Vec<CharReadParam>) -> ReadValueResult {
        let types: Vec<HapType> = params.iter()
            .map(|i| i.ctag.clone())
            .collect();
        info!("read_chars_value:{:?}", types);
        let mut result = vec![];
        let mut cids = vec![];
        let mut ids = vec![];
        for param in params.into_iter() {
            if let Some(stag) = param.stag {
                match self.mapping.get_by_left(&Cid::new(stag.clone(), MappingHapType::from(param.ctag))) {
                    None => {
                        warn!("no mapping for stag:{:?},ctag:{:?}",stag, param.ctag);
                        result.push(CharReadResult {
                            cid: param.cid,
                            success: true,
                            value: None,
                        });
                    }
                    Some(aid) => {
                        cids.push(param.cid);
                        ids.push(aid.clone());
                    }
                }
            }
        }

        if !ids.is_empty() {
            let results = self.ctx.dev
                .as_miot_device()?
                .read_properties(ids).await?;
            if results.len() != cids.len() {
                return Err(anyhow!("update result length not equal to cids length"));
            }
            for (i, cid) in cids.into_iter().enumerate() {
                let value = results.get(i)
                    .and_then(|v| v.value.clone())
                    .and_then(|v| v.as_bool())
                    .ok_or(anyhow!("on value is none"))?;
                result.push(CharReadResult {
                    cid,
                    success: true,
                    value: Some(JsonValue::Bool(value)),
                });
            }
        };

        Ok(result)
    }

    async fn update_chars_value(&self, params: Vec<CharUpdateParam>) -> UpdateValueResult {
        let types: Vec<(HapType, JsonValue, JsonValue)> = params.iter()
            .map(|i| (i.ctag.clone(), i.old_value.clone(), i.new_value.clone()))
            .collect();
        info!("update value:{:?}", types);
        let mut result = vec![];
        let mut cids = vec![];
        let mut ids = vec![];
        for param in params.into_iter() {
            if let Some(stag) = param.stag {
                match self.mapping.get_by_left(&Cid::new(stag.clone(), param.ctag)) {
                    None => {
                        warn!("no mapping for stag:{:?},ctag:{:?}",stag, param.ctag);
                        result.push(CharUpdateResult {
                            cid: param.cid,
                            success: false,
                        });
                    }
                    Some(aid) => {
                        cids.push(param.cid);
                        ids.push((aid.clone(), param.new_value));
                    }
                }
            }
        }
        if !ids.is_empty() {
            let results = self.ctx.dev
                .as_miot_device()?
                .set_properties(ids).await?;
            if results.len() != cids.len() {
                return Err(anyhow!("update result length not equal to cids length"));
            }

            for (i, cid) in cids.into_iter().enumerate() {
                // let value = results.get(i)
                //     .and_then(|v| v.value.clone())
                //     .and_then(|v| v.as_bool())
                //     .ok_or(anyhow!("on value is none"))?;
                result.push(CharUpdateResult {
                    cid,
                    success: true,
                });
            }
        };

        Ok(result)
    }
}