use anyhow::anyhow;
use hex::FromHex;
use log::{error, info, trace};
use packed_struct::PackedStruct;
use serde_json::{json, Value};
use tokio::sync::RwLock;

use hl_integration::error::DeviceExitError;
use hl_integration::event::{EventListener, HlDeviceListenable};
use hl_integration::event::emitter::DeviceEventEmitter;
use hl_integration::hl_device::{HlDevice, RetryInfo};
use hl_integration::HlSourceDevice;
use hl_integration::platform::hap::hap_device;
use hl_integration::platform::hap::hap_device::HapDevice;
use xiaomi_ble_packet::ble_value_type::{BleValue, MiBleValueType, ValueLsbI16};
use crate::device::ble::value_types::BleValues;

use crate::device::common::emitter::MijiaEvent;
use crate::device::common::utils::get_hap_device_info;
use crate::device::miot_spec_device::{AsMiotDevice, DeviceInfo, MiotDeviceType, MiotSpecDevice};
use crate::proto::protocol::ExitError::NotGateway;
use crate::proto::protocol::RecvMessage;


/// https://tasmota.github.io/docs/Bluetooth/
/// 低功耗蓝牙设备
/// 目前低功耗蓝牙设备只支持被动获取数据
///spec_map 用户


pub struct BleDevice<T: AsMiotDevice> {
    pub info: DeviceInfo,
    pub(crate) emitter: DeviceEventEmitter,
    pub retry_info: RetryInfo,
    pub values: RwLock<BleValues>,
    gateway: T,
}


#[async_trait::async_trait]
impl<T: AsMiotDevice> HlDevice for BleDevice<T> {
    fn dev_id(&self) -> String {
        self.info.did.clone()
    }

    fn device_type(&self) -> &str {
        MiotDeviceType::Ble.as_ref()
    }

    async fn run(&self) -> Result<(), Box<dyn DeviceExitError>> {
        let mut recv = self.gateway
            .as_miot_device()
            .map_err(|_| NotGateway)?
            .get_event_recv()
            .await;

        while let Ok(MijiaEvent::GatewayMsg(msg)) = recv.recv().await {
            /// 收到数据
            trace!("收到网关数据:{:?}", msg);
            let data = msg.get_json_data();
            if let Some(method) = data.get("method") {
                //异步蓝牙事件
                if method.as_str() == Some("_async.ble_event") {
                    if let Some(v) = data.get("params") {
                        let did = v.as_object()
                            .and_then(|i| i.get("dev"))
                            .and_then(|i| i.as_object())
                            .and_then(|i| i.get("did"))
                            .and_then(|i| i.as_str());

                        if Some(self.info.did.as_str()) == did {
                            //获取evt
                            self.set_value_from_param(v).await;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn enabled(&self) -> bool {
        todo!()
    }

    fn retry_info(&self) -> &RetryInfo {
        &self.retry_info
    }
}

#[async_trait::async_trait]
impl<T: AsMiotDevice> HlDeviceListenable for BleDevice<T> {
    async fn add_listener(&self, listener: EventListener) -> i64 {
        self.emitter.add_listener(listener).await
    }

    fn remove_listener(&self, id: i64) -> i64 {
        self.emitter.remove_listener(id)
    }
}


impl<T: AsMiotDevice + 'static> HlSourceDevice for BleDevice<T> {}

impl<T: AsMiotDevice> HapDevice for BleDevice<T> {
    fn get_hap_info(&self) -> hap_device::DeviceInfo {
        get_hap_device_info(&self.info)
    }
}


impl<T: AsMiotDevice> BleDevice<T> {
    pub fn new(info: DeviceInfo, gateway: T) -> Self {
        Self {
            info,
            emitter: Default::default(),
            retry_info: Default::default(),
            values: Default::default(),
            gateway,
        }
    }

    async fn set_value_from_param(&self, param: &Value) {
        let evt_vec = param.as_object()
            .and_then(|i| i.get("evt"))
            .and_then(|i| i.as_array());
        if let Some(data) = evt_vec {
            let mut ble_value = BleValues::default();
            // 数据列表
            for val in data {
                //处理数据
                if let Some(eid) = val.get("eid").and_then(|i| i.as_u64()) {
                    if let Some(edata) = val.get("edata")
                        .and_then(|i| i.as_str())
                        .and_then(|i| <Vec<u8>>::from_hex(i.as_bytes()).ok()) {
                        //tid,edata
                        match Self::mapping_value(eid as u16, edata) {
                            Ok((tp, val)) => {
                                ble_value.set_value(tp, val);
                            }
                            Err(err) => {
                                error!("解析蓝牙事件数据错误错误:{:?},eid:0x{:x}", err, eid);
                            }
                        };
                    }
                }
            }
            //  数据提交
            self.values.write().await.extend(ble_value.clone());
        }
    }

    /// 值映射
    fn mapping_value(eid: u16, edata: Vec<u8>) -> anyhow::Result<(MiBleValueType, BleValue)> {
        match MiBleValueType::try_from(eid) {
            Ok(tp) => {
                let value = tp.unpack(edata.as_slice())?;
                Ok((tp, value))
            }
            Err(_) => {
                Err(anyhow!("未知的蓝牙事件: eid:0x{:x},edata:{:?}", eid,edata))
            }
        }
    }

    pub async fn get_value(&self, value_type: MiBleValueType) -> Option<BleValue> {
        self.values.read().await.get_value(value_type)
    }
}

#[test]
pub fn test() {
    //8C00
    let token_bytes = <[u8; 2]>::from_hex("D002".as_bytes()).unwrap();
    let a = ValueLsbI16::unpack(&token_bytes).unwrap();
    println!("{:?}", a);
}