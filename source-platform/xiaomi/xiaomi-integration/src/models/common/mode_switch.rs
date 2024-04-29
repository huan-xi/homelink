use std::collections::HashMap;
use std::sync::Arc;
use anyhow::anyhow;
use log::{info, warn};
use serde_json::json;
use hl_integration::event::events::{DeviceEvent, DeviceEventPointer};
use hl_integration::JsonValue;
use miot_proto::device::miot_spec_device::{AsMiotDevice, MiotDeviceArc};
use miot_proto::proto::miio_proto::MiotSpecId;
use target_hap::delegate::{CharReadParam, CharReadResult, CharUpdateParam, CharUpdateResult};
use target_hap::delegate::model::{AccessoryModelExtConstructor, ContextPointer, HapModelExt, HapModelExtPointer, ReadValueResult, UpdateValueResult};
use target_hap::hap::characteristic::Format;
use target_hap::hap::HapType;
use target_hap::iot::characteristic_value::CharacteristicValue;

pub struct ModelExt {
    ctx: ContextPointer,

    dev: MiotDeviceArc,
    params: Params,
}

#[derive(Debug, serde::Deserialize)]
pub struct Params {
    /// 关闭
    pub on: MiotSpecId,
    pub mode: MiotSpecId,
    /// 模式值映射 tagRead->5
    pub mode_map: HashMap<String, u64>,
}

impl AccessoryModelExtConstructor for ModelExt {
    fn new(ctx: ContextPointer, params: Option<JsonValue>) -> anyhow::Result<HapModelExtPointer> {
        let params = params.ok_or(anyhow!("mode switch params is none"))?;
        let params: Params = serde_json::from_value(params)?;

        let dev = MiotDeviceArc(ctx.dev.clone());
        Ok(Arc::new(Self {
            ctx,
            dev,
            params,
        }))
    }
}


#[async_trait::async_trait]
impl HapModelExt for ModelExt {
    /// 读取属性

    async fn read_chars_value(&self, params: Vec<CharReadParam>) -> ReadValueResult {
        let types: Vec<HapType> = params.iter()
            .map(|i| i.ctag.clone())
            .collect();

        let results = self.dev
            .as_miot_device()?
            .read_properties(vec![self.params.on, self.params.mode]).await?;
        let on = results.get(0)
            .and_then(|v| v.value.clone())
            .and_then(|v| v.as_bool())
            .ok_or(anyhow!("on value is none"))?;

        let mode = results.get(1)
            .and_then(|v| v.value.clone())
            .and_then(|v| v.as_u64())
            .ok_or(anyhow!("mode is none"))?;

        info!("read_chars_value:{:?}", types);
        let mut result = vec![];
        for param in params.into_iter() {
            let value = match (param.ctag, param.stag.clone()) {
                (HapType::PowerState, stag) => {
                    //如果on=false,模式全部为false
                    match (on, self.params.mode_map.get(stag.as_str())) {
                        (false, _) => {
                            Some(JsonValue::Bool(false))
                        }
                        (true, Some(v)) if *v == mode => {
                            //todo 将其余全部设为false
                            Some(JsonValue::Bool(true))
                        }
                        (true, Some(_)) => {
                            // 未匹配到,或者不等于当前模式
                            Some(JsonValue::Bool(false))
                        }
                        _ => {
                            None
                        }
                    }
                }
                _ => {
                    warn!("未处理type{:?}",param.ctag);
                    None
                }
            };


            let value = match param.ctag {
                HapType::PowerState => {
                    //如果on=false,模式全部为false
                    if on {
                        if let Some(v) = self.params.mode_map.get(param.stag.as_str()) {
                            let v = *v == mode;
                            Some(JsonValue::Bool(v))
                        } else {
                            None
                        }
                    } else {
                        Some(JsonValue::Bool(false))
                    }
                }
                // 读取模式开关的状态
                _ => None,
            };
            result.push(CharReadResult {
                sid: param.sid,cid: param.cid,
                success: true,
                value,
            });
        }
        info!("read_chars_value:{:?}", result);
        Ok(result)
    }

    async fn update_chars_value(&self, params: Vec<CharUpdateParam>) -> UpdateValueResult {
        let types: Vec<(HapType, JsonValue, JsonValue)> = params.iter()
            .map(|i| (i.ctag.clone(), i.old_value.clone(), i.new_value.clone()))
            .collect();
        info!("update_value:{:?}", types);
        let mut result = vec![];
        for param in params {
            match (param.ctag, param.stag) {
                (HapType::PowerState, stag) => {
                    let value = CharacteristicValue::try_format(Format::Bool, param.new_value)?
                        .value.as_bool()
                        .ok_or(anyhow!("power state value is none"))?;
                    if value {

                        //开启,将 mode 属性设置到当前值,
                        //并且将其他属性设置为false
                        /*    let mode = self.dev.read_property(self.params.mode).await?
                                .value
                                .as_u64()
                                .ok_or(anyhow!("mode value is none"))?;*/
                        // let mode = mode.to_string();
                        let mode = self.params.mode_map.get(stag.as_str())
                            .ok_or(anyhow!("mode value is none"))?;
                        //todo 其余开关全部设置成false
                        let result = self.dev
                            .as_miot_device()?
                            .set_property(self.params.mode, json!(mode)).await?;
                        info!("update_chars_value:{:?}", result);
                    } else {
                        ///直接设置成false
                        let result = self.dev
                            .as_miot_device()?
                            .set_properties(vec![
                                (self.params.on, JsonValue::Bool(false)),
                            ]).await?;
                        info!("update_chars_value:{:?}", result);
                    }
                }
                _ => {}
            }
            result.push(CharUpdateResult {
                cid: param.cid,
                success: true,
            })
        }

        Ok(result)
    }


    async fn on_event(&self, event_type: DeviceEventPointer) {
        info!("on_event:{:?}", event_type);
    }
}