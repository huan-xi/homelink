use std::sync::Arc;
use async_trait::async_trait;
use log::info;
use hl_integration::JsonValue;
use miot_proto::device::miot_spec_device::MiotDeviceArc;
use target_hap::delegate::{CharReadParam, CharReadResult, CharUpdateParam, CharUpdateResult};
use target_hap::delegate::model::{AccessoryModelExtConstructor, ContextPointer, HapModelExt, HapModelExtPointer, ReadValueResult, UpdateValueResult};
use target_hap::hap::HapType;

pub struct ModelExt {
    ctx: ContextPointer,
}

impl AccessoryModelExtConstructor for ModelExt {
    fn new(ctx: ContextPointer, params: Option<JsonValue>) -> anyhow::Result<HapModelExtPointer> {
        Ok(Arc::new(Self {
            ctx,
        }))
    }
}

#[async_trait::async_trait]
impl HapModelExt for ModelExt {
    async fn read_chars_value(&self, params: Vec<CharReadParam>) -> ReadValueResult {
        let mut res = vec![];
        info!("read_chars_value:{:?}",params);

        for c in params.into_iter() {
            match c.ctag {
                HapType::PowerState => {
                    res.push(CharReadResult::success(&c, Some(serde_json::Value::Bool(true))));
                }
                _ => {
                    todo!("未处理")
                }
            }
        }
        Ok(res)
    }

    async fn update_chars_value(&self, params: Vec<CharUpdateParam>) -> UpdateValueResult {
        let mut res = vec![];
        for c in params.into_iter() {
            match c.ctag {
                HapType::PowerState => {
                    res.push(CharUpdateResult {
                        cid: c.cid,
                        success: true,
                    });
                }
                _ => {
                    todo!("未处理")
                }
            }
        }
        Ok(res)
    }
}
