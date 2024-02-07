use std::sync::Arc;
use hap::characteristic::{CharReadParam, CharReadResult, CharUpdateParam, CharUpdateResult};
use hap::HapType;
use log::info;
use miot_spec::proto::miio_proto::MiotSpecId;
use crate::hap::models::{AccessoryModelExt, AccessoryModelExtConstructor, AccessoryModelExtPointer, ContextPointer, ReadValueResult, UpdateValueResult};
use sea_orm::JsonValue;

pub struct ModelExt {
    on: MiotSpecId,
}

impl Default for ModelExt {
    fn default() -> Self {
        Self {
            on: MiotSpecId::new(2, 1),

        }
    }
}

impl AccessoryModelExtConstructor for ModelExt {
    fn new(ctx: ContextPointer, params: Option<JsonValue>) -> anyhow::Result<AccessoryModelExtPointer> {
        Ok(Arc::new(Self {}))
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
            let value = match param.ctag {
                _ => None,
            };
            result.push(CharReadResult {
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
            result.push(CharUpdateResult {
                cid: param.cid,
                success: true,
            })
        }

        Ok(result)
    }
}