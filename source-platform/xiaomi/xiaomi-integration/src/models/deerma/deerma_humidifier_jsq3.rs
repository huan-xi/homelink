use std::sync::Arc;

use log::info;
use serde_json::json;

use hl_integration::JsonValue;
use miot_proto::device::miot_spec_device::AsMiotDevice;
use miot_proto::proto::miio_proto::MiotSpecId;
use target_hap::delegate::{CharReadParam, CharReadResult, CharUpdateParam, CharUpdateResult};
use target_hap::delegate::model::{AccessoryModelExtConstructor, ContextPointer, HapModelExt, HapModelExtPointer, ReadValueResult, UpdateValueResult};
use target_hap::hap::characteristic::Format;
use target_hap::hap::HapType;
use target_hap::iot::characteristic_value::CharacteristicValue;

///加湿器
pub struct ModelExt {
    ctx: ContextPointer,
    on: MiotSpecId,
    target_humidity: MiotSpecId,
}

impl AccessoryModelExtConstructor for ModelExt {
    fn new(ctx: ContextPointer, params: Option<JsonValue>) -> anyhow::Result<HapModelExtPointer> {
        Ok(Arc::new(
            Self {
                ctx,
                on: MiotSpecId::new(2, 1),
                target_humidity: MiotSpecId::new(2, 6),
            }
        ))
    }
}


/// homekit 有除湿模式,设备没有
/// 配件有档位, 映射成homekit 的开关
#[async_trait::async_trait]
impl HapModelExt for ModelExt {
    ///[CurrentRelativeHumidity
    /// , TargetHumidifierDehumidifierState,
    /// CurrentHumidifierDehumidifierState, Active]
    async fn read_chars_value(&self, params: Vec<CharReadParam>) -> ReadValueResult {
        let types: Vec<HapType> = params.iter()
            .map(|i| i.ctag.clone())
            .collect();
        //读取值
        let values = self.ctx.dev
            .as_miot_device()?
            .read_properties(vec![self.on, self.target_humidity]).await?;
        let on = values.get(0);
        let target_humidity = values.get(1);
        info!("read_chars_value:{:?}", types);
        let mut result = vec![];
        for param in params.into_iter() {
            let value = match param.ctag {
                HapType::CurrentRelativeHumidity => {
                    //湿度
                    target_humidity
                        .and_then(|i| i.value.clone())
                }
                HapType::TargetHumidifierDehumidifierState | HapType::CurrentHumidifierDehumidifierState => {

                    // pub enum Value {
                    //     Inactive = 0,
                    //     Idle = 1,
                    //     Humidifying = 2,
                    //     Dehumidifying = 3,
                    // }
                    // CurrentHumidifierDehumidifierStateCharacteristic
                    Some(json!(1))
                }
                HapType::Active => {
                    Some(json!(true))
                }
                _ => {
                    None
                }
            };
            result.push(CharReadResult {
                cid: param.cid,
                success: true,
                value,
            })
        }
        Ok(result)
    }

    async fn update_chars_value(&self, params: Vec<CharUpdateParam>) -> UpdateValueResult {
        let ctx = &self.ctx;
        let types: Vec<(HapType, JsonValue, JsonValue)> = params.iter()
            .map(|i| (i.ctag.clone(), i.old_value.clone(), i.new_value.clone()))
            .collect();
        info!("update value:{:?}", types);
        let mut result = vec![];
        for param in params {
            match param.ctag {
                HapType::Active => {
                    let val = CharacteristicValue::format(Format::Bool, param.new_value);
                    ctx.dev.set_property(self.on, val.value).await?;
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
}