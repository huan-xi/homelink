use std::collections::HashMap;
use std::sync::Arc;
use anyhow::anyhow;
/// 开,将 mode 属性设置到当前值,
/// 并且将其他属性设置为false,关则将其他设置成false

use hap::characteristic::{CharReadParam, CharUpdateParam, Format, CharReadResult, CharUpdateResult};
use hap::HapType;
use log::{error, info, warn};
use miot_spec::proto::miio_proto::MiotSpecId;
use crate::hap::models::{AccessoryModelExt, AccessoryModelExtConstructor, AccessoryModelExtPointer, ContextPointer, PARAM_KEY, ReadValueResult, UpdateValueResult};
use sea_orm::JsonValue;
use serde_json::json;
use hl_device::event::events::DeviceEvent;
use miot_spec::device::common::emitter::EventType;
use crate::hap::iot::iot_characteristic::CharacteristicValue;

pub struct ModelExt {
    ctx: ContextPointer,
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
    fn new(ctx: ContextPointer, params: Option<JsonValue>) -> anyhow::Result<AccessoryModelExtPointer> {
        let params = params.ok_or(anyhow!("mode switch params is none"))?;
        let params: Params = serde_json::from_value(params)?;

        Ok(Arc::new(Self {
            ctx,
            params,
        }))
    }
}


#[async_trait::async_trait]
impl AccessoryModelExt for ModelExt {
    /// 读取属性

    async fn read_chars_value(&self, params: Vec<CharReadParam>) -> ReadValueResult {
        let types: Vec<HapType> = params.iter()
            .map(|i| i.ctag.clone())
            .collect();

        let results = self.ctx.dev
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
                (HapType::PowerState, Some(stag)) => {
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
                (_, None) => {
                    warn!("service id:{},无tag",param.sid);
                    None
                }
                _ => {
                    warn!("未处理type{:?}",param.ctag);
                    None
                }
            };


            let value = match param.ctag {
                // 读取模式开关的状态
                HapType::PowerState => {
                    //如果on=false,模式全部为false
                    if on {
                        if let Some(v) = self.params.mode_map.get(param.stag
                            .clone()
                            .ok_or(anyhow!("service tag is none"))?.as_str()) {
                            let v = *v == mode;
                            Some(JsonValue::Bool(v))
                        } else {
                            None
                        }
                    } else {
                        Some(JsonValue::Bool(false))
                    }
                }
                _ => None,
            };
            result.push(CharReadResult {
                cid: param.cid,
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
                (HapType::PowerState, Some(stag)) => {
                    let value = CharacteristicValue::try_format(Format::Bool, param.new_value)?
                        .value.as_bool()
                        .ok_or(anyhow!("power state value is none"))?;
                    if value {

                        //开启,将 mode 属性设置到当前值,
                        //并且将其他属性设置为false
                        /*    let mode = self.ctx.dev.read_property(self.params.mode).await?
                                .value
                                .as_u64()
                                .ok_or(anyhow!("mode value is none"))?;*/
                        // let mode = mode.to_string();
                        let mode = self.params.mode_map.get(stag.as_str())
                            .ok_or(anyhow!("mode value is none"))?;
                        //todo 其余开关全部设置成false
                        let result = self.ctx.dev
                            .set_property(self.params.mode, json!(mode)).await?;
                        info!("update_chars_value:{:?}", result);
                    } else {
                        ///直接设置成false
                        let result = self.ctx.dev
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

    async fn on_event(&self, event_type: DeviceEvent) {
        /*if let EventType::UpdatePropertyBatch(values) = event_type {
            for id in values {
                /*if id.siid == self.model.siid && id.piid == self.model.piid {
                    //处理
                };*/
            }
        };*/
    }
}