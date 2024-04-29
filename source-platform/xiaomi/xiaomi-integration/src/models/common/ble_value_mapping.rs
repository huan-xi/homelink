use std::sync::Arc;
use anyhow::anyhow;

use log::{debug, info};
use serde_json::json;
use hl_integration::{JsonValue, SourceDevicePointer};
use miot_proto::device::ble::ble_device::BleDevice;
use miot_proto::device::miot_spec_device::MiotDeviceArc;
use target_hap::delegate::{CharReadParam, CharReadResult, CharUpdateParam, CharUpdateResult};
use target_hap::delegate::model::{AccessoryModelExtConstructor, ContextPointer, HapModelExt, HapModelExtPointer, ReadValueResult, UpdateValueResult};
use target_hap::hap::HapType;
use xiaomi_ble_packet::ble_value_type::MiBleValueType;

/// 通过值类型获取 ble 设备上的值

pub struct BleDeviceWrapper {
    pub dev: SourceDevicePointer,
}

impl BleDeviceWrapper {
    pub fn as_ble_device(&self) -> anyhow::Result<&BleDevice<MiotDeviceArc>> {
        self.dev.downcast_ref::<BleDevice<MiotDeviceArc>>()
            .ok_or(anyhow!("设备不是ble 设备"))
    }
}

pub struct ModelExt {
    ctx: ContextPointer,
    dev: BleDeviceWrapper,
}


impl AccessoryModelExtConstructor for ModelExt {
    fn new(ctx: ContextPointer, _params: Option<JsonValue>) -> anyhow::Result<HapModelExtPointer> {
        let dev = BleDeviceWrapper {
            dev: ctx.dev.clone(),
        };
        let _ = dev.as_ble_device()?;
        Ok(Arc::new(Self { ctx, dev }))
    }
}


#[async_trait::async_trait]
impl HapModelExt for ModelExt {
    fn is_subscribe_event(&self) -> bool { false }
    async fn read_chars_value(&self, params: Vec<CharReadParam>) -> ReadValueResult {
        let types: Vec<HapType> = params.iter()
            .map(|i| i.ctag.clone())
            .collect();
        debug!("read_chars_value:{:?}", types);
        let mut result = vec![];
        let dev = self.dev.as_ble_device()?;

        for param in params.into_iter() {
            let value = match param.ctag {
                HapType::CurrentTemperature => {
                    let value = dev
                        .get_value(MiBleValueType::Temperature)
                        .await;
                    value.map(|v| json!(v.as_u64() as f32/ 10.0))
                }
                HapType::CurrentRelativeHumidity => {
                    let value = dev
                        .get_value(MiBleValueType::Humidity)
                        .await;
                    value.map(|v| json!(v.as_u64() as f32/ 10.0))
                }
                _ => None,
            };
            result.push(CharReadResult {
                sid: param.sid,
                cid: param.cid,
                success: value.is_some(),
                value,
            });
        }
        debug!("ble_value_mapping read_chars_value result:{:?}", result);
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