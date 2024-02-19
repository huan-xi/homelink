use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use anyhow::anyhow;
use bimap::BiMap;
use super::super::types::{CharReadParam, CharReadResult, CharUpdateParam, CharUpdateResult};
use hap::HapType;
use log::{info, warn};
use crate::hap::models::{AccessoryModelExt, AccessoryModelExtConstructor, AccessoryModelExtPointer, ContextPointer, ReadValueResult, UpdateValueResult};
use sea_orm::JsonValue;
use serde_json::json;
use tap::TapFallible;
use ble_monitor::parse_advertisement::BlePlatform;
use hl_integration::event::events::DeviceEvent;
use xiaomi_ble_packet::ble_value_type::MiBleValueType;
use crate::device::native_ble::native_ble_device::{PropData};
use crate::hap::hap_type::MappingHapType;
use crate::hap::models::types::{CharIdentifier, DefaultStag};

#[derive(Debug, serde::Deserialize)]
pub struct ParamMapping {
    stag: String,
    ctag: MappingHapType,

    /// 类型字符串
    type_str: Option<String>,

    // etype: u16,
    // /// 单位转换器
    // convert: u8,
}

#[derive(Debug, serde::Deserialize)]
pub struct Param {
    platform: BlePlatform,
    mappings: Vec<ParamMapping>,
}


pub struct MappingParam {
    /// 类型
    etype: u16,
    /// 单位转换器
    convert: u8,
}

pub struct ModelExt {
    ctx: ContextPointer,
    platform: BlePlatform,
    //char,etype
    //etype,charId,
    char_mapping: BiMap<Arc<CharIdentifier>, u16>,
    convert_mapping: HashMap<u16, MappingParam>,
}


impl AccessoryModelExtConstructor for ModelExt {
    fn new(ctx: ContextPointer, params: Option<JsonValue>) -> anyhow::Result<AccessoryModelExtPointer> {
        let param: Param = serde_json::from_value(params.ok_or(anyhow!("参数不能为空"))?)?;
        let mut char_mapping = BiMap::new();
        for mapping in param.mappings.iter() {
            let char_id = Arc::new(CharIdentifier {
                stag: mapping.stag.clone(),
                ctag: mapping.ctag.clone(),
            });

            let mut etype = None;
            if let Some(type_str) = mapping.type_str.as_ref() {
                etype = Some(MiBleValueType::from_str(type_str.as_str())?);
            }
            let etype = etype.ok_or(anyhow!("etype not found"))?;
            // let etype=
            char_mapping.insert(char_id, etype.into());
        }

        Ok(Arc::new(Self {
            ctx,
            platform: param.platform,
            char_mapping,
            convert_mapping: Default::default(),
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
        for param in params.into_iter() {
            let id = CharIdentifier::from(&param);
            let etype = self.char_mapping.get_by_left(&id);

            if let Some(etype) = etype {
                let value = self.ctx.dev.as_native_ble()?
                    .value_register.get(&etype)
                    .map(|v| v.clone());

                if let Some(value) = value {
                    let value = self
                        .unpack_value(*etype, value)
                        .tap_err(|e| warn!("蓝牙数据 unpack value error:{:?}", e))
                        .ok();
                    result.push(CharReadResult {
                        cid: param.cid,
                        success: true,
                        value,
                    });
                } else {
                    result.push(CharReadResult::fail(param.cid));
                }
            } else {
                warn!("char not found:{},{:?}",param.stag, param.ctag);
                result.push(CharReadResult::fail(param.cid));
            }
        }
        Ok(result)
    }
    async fn update_chars_value(&self, params: Vec<CharUpdateParam>) -> UpdateValueResult {
        let types: Vec<(HapType, JsonValue, JsonValue)> = params.iter()
            .map(|i| (i.ctag.clone(), i.old_value.clone(), i.new_value.clone()))
            .collect();
        info!("update value:{:?}", types);
        let mut result = vec![];
        for param in params {
            match param.ctag {
                _ => {}
            }
            result.push(CharUpdateResult {
                cid: param.cid,
                success: true,
            })
        }

        Ok(result)
    }

    async fn on_event(&self, event_type: DeviceEvent) {
        if let Err(e) = self.on_event0(event_type).await {
            info!("on_event error:{:?}",e);
        }
    }
}

impl ModelExt {
    fn unpack_value(&self, etype: u16, edata: Vec<u8>) -> anyhow::Result<JsonValue> {
        return match self.platform {
            BlePlatform::Xiaomi => {
                /// 解码平台 解码
                let tp = MiBleValueType::try_from(etype)?;
                let value = tp.unpack(edata.as_slice())?;
                /// 转成hap 类型
                let value = value.as_u64();
                ///拿etype 对应的id
                /// 转换器转换
                let value = value as f32 / 10.0;
                info!("value:{:?}", value);
                Ok(json!(value))
            }
        };
        anyhow::bail!("not support platform:{:?}", self.platform);
    }

    async fn on_event0(&self, event_type: DeviceEvent) -> anyhow::Result<()> {
        if let DeviceEvent::PropertyChanged(data) = event_type {
            let data: PropData = serde_json::from_value(data)?;
            info!("on_event:{:?}", data);
            if let Some(char_id) = self.char_mapping.get_by_right(&data.etype) {
                let value = self.unpack_value(data.etype, data.edata)?;
                self.ctx.set_char_value(char_id.as_ref(), value).await;
            }
        };

        Ok(())
    }
}