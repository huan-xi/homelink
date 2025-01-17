use std::collections::HashMap;
use std::sync::Arc;

use anyhow::anyhow;
use bimap::BiMap;
use log::{debug, info, warn};

use hl_integration::JsonValue;
use miot_proto::device::miot_spec_device::{AsMiotDevice, MiotDeviceArc};
use miot_proto::proto::miio_proto::MiotSpecId;
use target_hap::delegate::{CharReadParam, CharReadResult, CharUpdateParam, CharUpdateResult};
use target_hap::delegate::model::{AccessoryModelExtConstructor, ContextPointer, HapModelExt, HapModelExtPointer, ReadValueResult, UpdateValueResult};
use target_hap::hap::HapType;
use target_hap::hap_type_wrapper::HapTypeWrapper;
use target_hap::iot::characteristic_value::CharacteristicValue;
use target_hap::types::CharIdentifier;

fn default_str() -> String {
    "default".to_string()
}

#[derive(Debug, serde::Deserialize)]
pub struct PropMappingParam {
    /// stag,ctag,{siid,piid}
    #[serde(default = "default_str")]
    stag: String,
    ctag: HapTypeWrapper,
    siid: i32,
    piid: i32,
}


/// 属性映射模型
pub struct ModelExt {
    ctx: ContextPointer,
    dev: MiotDeviceArc,
    mapping: BiMap<CharIdentifier, MiotSpecId>,

}


impl AccessoryModelExtConstructor for ModelExt {
    fn new(ctx: ContextPointer, params: Option<JsonValue>) -> anyhow::Result<HapModelExtPointer> {
        let params = params.ok_or(anyhow!("mode switch params is none"))?;

        let params: Vec<PropMappingParam> = serde_json::from_value(params)?;
        let mut mapping = BiMap::new();
        for param in params {
            let cid = CharIdentifier::new(param.stag, param.ctag);
            mapping.insert(cid.clone(), MiotSpecId::new(param.siid, param.piid));
        }

        let dev = MiotDeviceArc(ctx.dev.clone());
        Ok(Arc::new(Self {
            ctx,
            dev,
            mapping,
        }))
    }
}


#[async_trait::async_trait]
impl HapModelExt for ModelExt {
    async fn read_chars_value(&self, params: Vec<CharReadParam>) -> ReadValueResult {
        let types: Vec<HapType> = params.iter()
            .map(|i| i.ctag.clone())
            .collect();
        info!("read_chars_value:{:?}", types);
        let mut result = vec![];
        let mut read_params = vec![];
        let mut ids = vec![];
        for param in params.into_iter() {
            let stag = param.stag.clone();
            let cid = CharIdentifier::new(stag.clone(), param.ctag);
            match self.mapping.get_by_left(&cid) {
                None => {
                    warn!("no mapping for stag:{:?},ctag:{:?}",stag, param.ctag);
                    result.push(CharReadResult {
                        sid: param.sid,
                        cid: param.cid,
                        success: true,
                        value: None,
                    });
                }
                Some(aid) => {
                    read_params.push(param);
                    ids.push(aid.clone());
                }
            }
        }

        if !ids.is_empty() {
            if !self.dev.is_running().await? {
                //全部失败
                debug!("device is not running");
                for param in read_params.into_iter() {
                    result.push(CharReadResult::fail(&param));
                }
                return Ok(result);
            }

            let results = self
                .dev
                .read_properties(ids).await?;

            if results.len() != read_params.len() {
                return Err(anyhow!("update result length not equal to cids length"));
            }
            for (i, param) in read_params.into_iter().enumerate() {
                match results.get(i) {
                    None => {
                        result.push(CharReadResult::fail(&param));
                    }
                    Some(val) => {
                        let value = val.value.clone();
                        //格式化
                        let mut value = value
                            .map(|v| CharacteristicValue::format(param.format, v).value);
                        //转换
                        if let Some(val) = value.clone() {
                            if let Some(c) = self.ctx.convertor_map.get(&CharIdentifier::new(param.stag.clone(), param.ctag)) {
                                if let Ok(val) = c.ext.from(val) {
                                    value = Some(val);
                                }
                            }
                        }
                        result.push(CharReadResult::success(&param, value));
                    }
                }
            }
        };
        info!("read_chars_value result:{:?}", result);
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
            let stag = param.stag;
            match self.mapping.get_by_left(&CharIdentifier::new(stag.clone(), param.ctag)) {
                None => {
                    warn!("no mapping for stag:{:?},ctag:{:?}",stag, param.ctag);
                    result.push(CharUpdateResult {
                        cid: param.cid,
                        success: false,
                    });
                }
                Some(aid) => {
                    cids.push(param.cid);
                    let mut value = CharacteristicValue::try_format(param.format, param.new_value)?.value;
                    //转换
                    if let Some(c) = self.ctx.convertor_map.get(&CharIdentifier::new(stag.clone(), param.ctag)) {
                        if let Ok(val) = c.ext.to(value.clone()) {
                            value = val
                        }
                    }

                    ids.push((aid.clone(), value));
                }
            }
        }
        if !ids.is_empty() {
            let results = self.dev
                .as_miot_device()?
                .set_properties(ids).await?;
            if results.len() != cids.len() {
                return Err(anyhow!("update result length not equal to cids length"));
            }

            for (i, cid) in cids.into_iter().enumerate() {
                result.push(CharUpdateResult {
                    cid,
                    success: true,
                });
            }
        };

        Ok(result)
    }
}