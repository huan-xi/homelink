use std::sync::Arc;

use log::{info};
use hl_integration::JsonValue;
use target_hap::delegate::{CharReadParam, CharReadResult, CharUpdateParam, CharUpdateResult};
use target_hap::delegate::model::{AccessoryModelExtConstructor, ContextPointer, HapModelExt, HapModelExtPointer, ReadValueResult, UpdateValueResult};
use target_hap::hap::HapType;


pub struct ModelExt {
    ctx: ContextPointer,
}


impl AccessoryModelExtConstructor for ModelExt {
    fn new(ctx: ContextPointer, params: Option<JsonValue>) -> anyhow::Result<HapModelExtPointer> {
        Ok(Arc::new(Self { ctx }))
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
        for param in params.into_iter() {
            let value = match param.ctag {
                _ => None,
            };
            result.push(CharReadResult {
                sid: param.sid,cid: param.cid,
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
                sid: param.sid,cid: param.cid,
                success: true,
            })
        }

        Ok(result)
    }
}