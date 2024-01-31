use hap::characteristic::{CharReadParam, CharUpdateParam, Format, ReadCharValue, UpdateCharValue};
use hap::HapType;
use log::info;
use sea_orm::JsonValue;
use serde_json::json;
use hap::characteristic::active::ActiveCharacteristic;
use hap::characteristic::active_identifier::ActiveIdentifierCharacteristic;
use hap::characteristic::current_humidifier_dehumidifier_state::CurrentHumidifierDehumidifierStateCharacteristic;
use miot_spec::proto::miio_proto::MiotSpecId;
use crate::hap::iot::iot_characteristic::CharacteristicValue;
use crate::hap::models::{AccessoryModelExt, ContextPointer, ReadValueResult, UpdateValueResult};

///加湿器
pub struct ModelExt {
    on: MiotSpecId,
    target_humidity: MiotSpecId,
}

impl Default for ModelExt {
    fn default() -> Self {
        Self {
            on: MiotSpecId::new(2, 1),
            target_humidity: MiotSpecId::new(2, 6),

        }
    }
}

/// homekit 有除湿模式,设备没有
/// 配件有档位, 映射成homekit 的开关
#[async_trait::async_trait]
impl AccessoryModelExt for ModelExt {
    ///[CurrentRelativeHumidity
    /// , TargetHumidifierDehumidifierState,
    /// CurrentHumidifierDehumidifierState, Active]
    async fn read_chars_value(&self, ctx: ContextPointer, params: Vec<CharReadParam>) -> ReadValueResult {
        let types: Vec<HapType> = params.iter()
            .map(|i| i.ctag.clone())
            .collect();
        //读取值
        let values = ctx.dev.read_properties(vec![self.on, self.target_humidity]).await?;
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
            result.push(ReadCharValue {
                cid: param.cid,
                success: true,
                value,
            })
        }
        Ok(result)
    }

    async fn update_chars_value(&self, ctx: ContextPointer, params: Vec<CharUpdateParam>) -> UpdateValueResult {
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

            result.push(UpdateCharValue {
                cid: param.cid,
                success: true,
            })
        }

        Ok(result)
    }
}