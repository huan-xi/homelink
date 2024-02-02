use std::sync::Arc;
use anyhow::anyhow;
/// 开,将 mode 属性设置到当前值,
/// 并且将其他属性设置为false,关则将其他设置成false

use hap::characteristic::{CharReadParam, CharUpdateParam, ReadCharValue, UpdateCharValue};
use hap::HapType;
use log::info;
use miot_spec::proto::miio_proto::MiotSpecId;
use crate::hap::models::{AccessoryModelExt, AccessoryModelExtConstructor, AccessoryModelExtPointer, ContextPointer, PARAM_KEY, ReadValueResult, UpdateValueResult};
use sea_orm::JsonValue;
use miot_spec::device::common::emitter::EventType;

pub struct ModelExt {}

pub struct Params {
    ctx: ContextPointer,
    /// 关闭
    pub on: MiotSpecId,
    pub model: MiotSpecId,
}

impl AccessoryModelExtConstructor for ModelExt {
    fn new(ctx: ContextPointer, params: Option<JsonValue>) -> anyhow::Result<AccessoryModelExtPointer> {
        Ok(Arc::new(Self {}))
    }
}


impl Default for ModelExt {
    fn default() -> Self {
        Self {
            // on: MiotSpecId::new(2, 1),
            // model: MiotSpecId::new(2, 5),
        }
    }
}


#[async_trait::async_trait]
impl AccessoryModelExt for ModelExt {

    /// 读取属性

    async fn read_chars_value(&self, params: Vec<CharReadParam>) -> ReadValueResult {
        let types: Vec<HapType> = params.iter()
            .map(|i| i.ctag.clone())
            .collect();


        // ctx.dev.read_properties(vec![self.on, self.model]);

        info!("read_chars_value:{:?}", types);
        let mut result = vec![];
        for param in params.into_iter() {
            let value = match param.ctag {
                _ => None,
            };
            result.push(ReadCharValue {
                cid: param.cid,
                success: true,
                value,
            });
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
            result.push(UpdateCharValue {
                cid: param.cid,
                success: true,
            })
        }

        Ok(result)
    }

    async fn on_event(&self, event_type: EventType) {
        if let EventType::UpdatePropertyBatch(values) = event_type {
            for id in values {
                /*if id.siid == self.model.siid && id.piid == self.model.piid {
                    //处理
                };*/
            }
        };
    }
}