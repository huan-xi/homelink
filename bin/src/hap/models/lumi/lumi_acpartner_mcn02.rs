use futures_util::future::ok;
use log::info;
use sea_orm::JsonValue;
use hap::characteristic::{CharReadParam, CharUpdateParam, ReadCharValue};
use hap::characteristic::target_heating_cooling_state::TargetHeatingCoolingStateCharacteristic;
use hap::HapType;
use miot_spec::proto::miio_proto::{MiotSpecDTO, MiotSpecId};
use crate::hap::models::{AccessoryModelExt, ContextPointer, ReadValueResult, UpdateValueResult};


pub struct ModelExt {
    on: MiotSpecId,
    model: MiotSpecId,
    ///Target Temperature 设置的温度
    target_temperature: MiotSpecId,
}

impl Default for ModelExt {
    fn default() -> Self {
        Self {
            on: MiotSpecId::new(2, 1),
            model: MiotSpecId::new(2, 2),
            target_temperature: MiotSpecId::new(2, 3),

        }
    }
}

/// 温度传感器?
///https://home.miot-spec.com/s/lumi.acpartner.mcn02
/// https://home.miot-spec.com/spec?type=urn:miot-spec-v2:device:air-conditioner:0000A004:lumi-mcn02:1
#[async_trait::async_trait]
impl AccessoryModelExt for ModelExt {
    /// [CharReadParam { sid: 13, stag: None, cid: 17, ctag: TargetTemperature },
    /// CharReadParam { sid: 13, stag: None, cid: 16, ctag: CurrentTemperature },
    /// CharReadParam { sid: 13, stag: None, cid: 14, ctag: CurrentHeatingCoolingState },
    /// CharReadParam { sid: 13, stag: None, cid: 18, ctag: TemperatureDisplayUnits },
    /// CharReadParam { sid: 13, stag: None, cid: 15, ctag: TargetHeatingCoolingState }]
    async fn read_chars_value(&self, ctx: ContextPointer, params: Vec<CharReadParam>) -> ReadValueResult {
        let types: Vec<HapType> = params.iter()
            .map(|i| i.ctag.clone())
            .collect();
        info!("read_chars_value:{:?}", types);

        let mut result = vec![];
        let values = ctx.dev.read_properties(vec![self.on, self.model, self.target_temperature]).await?;
        let on = values.get(0);
        let model = values.get(1);
        let target_temperature = values.get(2);

        for param in params.into_iter() {
            let value = match param.ctag {
                HapType::CurrentTemperature | HapType::TargetTemperature => {
                    target_temperature
                        .and_then(|i| i.value.clone())
                }
                HapType::CurrentHeatingCoolingState | HapType::TargetHeatingCoolingState => {
                    //模式转换
                    Self::get_model(on, model).map(|f| serde_json::json!(f))
                }
                HapType::TemperatureDisplayUnits => {
                    //显示单位
                    Some(serde_json::json!(0))
                }
                _ => {
                    None
                }
            };
            result.push(ReadCharValue {
                cid: param.cid,
                success: true,
                value,
            });
        }
        info!("read result:{:?}", result);


        Ok(result)
    }

    async fn update_chars_value(&self, ctx: ContextPointer, params: Vec<CharUpdateParam>) -> UpdateValueResult {
        let types: Vec<(HapType, JsonValue, JsonValue)> = params.iter()
            .map(|i| (i.ctag.clone(), i.old_value.clone(), i.new_value.clone()))
            .collect();
        info!("update value:{:?}", types);
        let results = vec![];
        for param in params {
            match param.ctag {
                HapType::TargetHeatingCoolingState => {
                    if param.new_value == 0 {
                        //调用设备关闭
                        ctx.dev.set_property(self.on, serde_json::json!(false)).await?;
                    } else {
                        //获取对应的模式
                        let model = Self::get_model_from_hap(param.new_value);
                        if let Some(model) = model {
                            ctx.dev.set_property(self.model, serde_json::json!(model)).await?;
                            ctx.dev.set_property(self.on, serde_json::json!(true)).await?;
                        }
                    }
                }
                HapType::TargetTemperature => {
                    ctx.dev.set_property(self.target_temperature, param.new_value.clone()).await?;
                    ctx.hap_manager.update_char_value_by_id(ctx.aid, param.sid, HapType::CurrentTemperature, param.new_value).await?;
                }
                _ => {}
            };
        }
        Ok(results)
    }
}

impl ModelExt {
    fn get_model_from_hap(value: JsonValue) -> Option<i64> {
        if let Some(val) = value.as_i64() {
            match val {
                1 => return Some(3),
                2 => return Some(1),
                3 => return Some(0),
                _ => {}
            }
        }

        return None;
    }
    ///hap Off = 0,Heat = 1,Cool = 2,Auto = 3,

    ///0 - Auto
    /// 1 - Cool
    /// 2 - Dry
    /// 3 - Heat
    /// 4 - Fan

    fn get_model(on: Option<&MiotSpecDTO>, model: Option<&MiotSpecDTO>) -> Option<i64> {
        if let (Some(on), Some(model)) = (on, model) {
            if on.value.clone().and_then(|f| f.as_bool()) == Some(false) {
                return Some(0);
            };
            if let Some(m) = model.value.clone().and_then(|f| f.as_i64()) {
                match m {
                    0 => return Some(3),
                    1 => return Some(2),
                    3 => return Some(1),
                    //这里没有对应的模式转成3
                    4 => return Some(3),
                    _ => {}
                }
                return Some(m);
            };
        };
        return None;
    }
}