use std::sync::Arc;
use anyhow::anyhow;
use log::{debug, info, warn};
use hl_integration::JsonValue;
use miot_proto::device::miot_spec_device::{AsMiotDevice, MiotDeviceArc};
use miot_proto::proto::miio_proto::{MiotSpecDTO, MiotSpecId};
use target_hap::delegate::{CharReadParam, CharReadResult, CharUpdateParam, CharUpdateResult};
use target_hap::delegate::model::{AccessoryModelExtConstructor, ContextPointer, HapModelExt, HapModelExtPointer, ReadValueResult, UpdateValueResult};
use target_hap::hap::HapType;


pub struct ModelExt {
    ctx: ContextPointer,
    arming_mode: MiotSpecId,
    dev: MiotDeviceArc,
}

impl AccessoryModelExtConstructor for ModelExt {
    fn new(ctx: ContextPointer, params: Option<JsonValue>) -> anyhow::Result<HapModelExtPointer> {
        let dev = MiotDeviceArc(ctx.dev.clone());
        Ok(Arc::new(Self {
            ctx,
            dev,
            arming_mode: MiotSpecId { siid: 3, piid: 1 },
        }))
    }
}


/// todo 4状态
///
///homekit 米家
///  0 在家  1.
///  1 离家  2
///  2 睡眠  3
///  3 停用  0
///
#[async_trait::async_trait]
impl HapModelExt for ModelExt {
    //[CharReadParam { sid: 13, stag: None, cid: 15, ctag: SecuritySystemTargetState },
    // CharReadParam { sid: 13, stag: None, cid: 14, ctag: SecuritySystemCurrentState }]
    async fn read_chars_value(&self, params: Vec<CharReadParam>) -> ReadValueResult {
        //读取设备属性值
        debug!("read values:{:?}", params);
        let value = self.dev.read_property(self.arming_mode.siid, self.arming_mode.piid).await?;
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
            result.push(CharReadResult::success(&param, value.clone()));
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
            self.dev.as_miot_device()?.set_property(self.arming_mode, serde_json::json!(value)).await?;
            results.push(CharUpdateResult {
                cid: param.cid,
                success: true,
            });
            ctx.hap_manager.update_char_value_by_id(ctx.aid, param.sid, HapType::SecuritySystemCurrentState, param.new_value).await?;
        }
        //info
        Ok(results)
    }
}