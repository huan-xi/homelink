use std::sync::Arc;
use anyhow::anyhow;
use log::{debug, info};
use sea_orm::JsonValue;
use hap::characteristic::{CharReadParam, CharUpdateParam, ReadCharValue, UpdateCharValue};
use hap::HapType;
use miot_spec::proto::miio_proto::MiotSpecId;
use crate::hap::models::{AccessoryModelExt, AccessoryModelExtConstructor, AccessoryModelExtPointer, ContextPointer, ReadValueResult, UpdateValueResult};

pub struct ModelExt {
    ctx: ContextPointer,
    arming_mode: MiotSpecId,
}

impl AccessoryModelExtConstructor for ModelExt {
    fn new(ctx: ContextPointer, params: Option<JsonValue>) -> anyhow::Result<AccessoryModelExtPointer> {
        Ok(Arc::new(Self {
            ctx,
            arming_mode: MiotSpecId { siid: 3, piid: 1 },
        }))
    }
}


/// todo 4状态
#[async_trait::async_trait]
impl AccessoryModelExt for ModelExt {
    //[CharReadParam { sid: 13, stag: None, cid: 15, ctag: SecuritySystemTargetState },
    // CharReadParam { sid: 13, stag: None, cid: 14, ctag: SecuritySystemCurrentState }]
    async fn read_chars_value(&self, params: Vec<CharReadParam>) -> ReadValueResult {
        //读取设备属性值
        debug!("read values:{:?}", params);
        let value = self.ctx.dev.read_property(self.arming_mode.siid, self.arming_mode.piid).await?;
        let value = value
            .and_then(|v| v.as_i64())
            .and_then(|v| {
                if v == 0 {
                    Some(3)
                } else {
                    Some(v - 1)
                }
            })
            .map(|v| JsonValue::from(v));
        let mut result = vec![];
        for param in params.into_iter() {
            result.push(ReadCharValue {
                cid: param.cid,
                success: true,
                value: value.clone(),
            });
        }
        Ok(result)
    }

    async fn update_chars_value(&self, params: Vec<CharUpdateParam>) -> UpdateValueResult {
        let ctx = &self.ctx;
        debug!("update values:{:?}", params);
        let mut results = vec![];
        for param in params.into_iter() {
            let value = param.new_value
                .as_i64()
                .ok_or(anyhow!("value is not i64"))?;
            //3对应的是米家的0
            let value = if value == 3 {
                0
            } else {
                value + 1
            };
            ctx.dev.set_property(self.arming_mode, serde_json::json!(value)).await?;
            results.push(UpdateCharValue {
                cid: param.cid,
                success: true,
            });
            ctx.hap_manager.update_char_value_by_id(ctx.aid, param.sid, HapType::SecuritySystemCurrentState, param.new_value).await?;
        }
        //info
        Ok(results)
    }
}