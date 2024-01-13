use std::mem;
use std::sync::{Arc};
use std::time::Duration;
use anyhow::anyhow;
use futures_util::future::{BoxFuture, join_all};
use crate::device::miot_spec_device::{BaseMiotSpecDevice, DeviceInfo, MiotDeviceType, MiotSpecDevice};
use futures_util::FutureExt;
use hex::FromHex;
use log::{debug, error, info, warn};
use num_enum::TryFromPrimitiveError;
use packed_struct::{PackedStruct, PackingError};
use serde_json::Value;
use tokio::sync::RwLock;
use crate::device::ble::value_types::{BleValue, BleValueType, ValueLsbI16};
use crate::device::gateway::gateway::OpenMiioGatewayDevice;
use crate::device::emitter::{DataListener, DataEmitter, ListenDateType};
use crate::device::MiotDevicePointer;
use crate::proto::miio_proto::MiotSpecProtocolPointer;
use crate::proto::protocol::{ExitError, RecvMessage};

/// https://tasmota.github.io/docs/Bluetooth/
/// 低功耗蓝牙设备
/// 目前低功耗蓝牙设备只支持被动获取数据



pub struct BleDevice {
    pub info: DeviceInfo,
    base: BaseMiotSpecDevice,
    gateway: MiotDevicePointer,
    // 值
    values: Arc<RwLock<BleValue>>,
    // 传输协议
    // proto: MiotSpecProtocolPointer,
}

impl MiotSpecDevice for BleDevice {
    fn get_info(&self) -> &DeviceInfo { &self.info }

    fn get_base(&self) -> &BaseMiotSpecDevice {
        &self.base
    }

    fn get_proto(&self) -> BoxFuture<Result<MiotSpecProtocolPointer, ExitError>> {
        self.gateway.get_proto()
    }
    /// 1.同步网关的状态
    fn run(&self) -> BoxFuture<Result<(), ExitError>> {
        async move {
            let gw_proto = self.gateway.get_proto().await?;
            let mut recv = gw_proto.recv();
            while let Ok(msg) = recv.recv().await {
                /// 收到数据
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
                                Self::get_value_from_param(v);
                            }
                        }
                    }
                }
            }
            Ok(())
        }.boxed()
    }

    fn get_device_type(&self) -> MiotDeviceType {
        return MiotDeviceType::Ble(self);
    }
}

impl BleDevice {
    pub fn new(info: DeviceInfo, gateway: Arc<OpenMiioGatewayDevice>) -> Self {
        //网关上设置一个监听器,监听属于我的消息
        //蓝牙协议

        Self {
            info,
            base: BaseMiotSpecDevice {
                ..BaseMiotSpecDevice::default()
            },
            gateway,
            values: Default::default(),
        }
    }

    fn get_value_from_param(param: &Value) -> Option<BleValue> {
        let evt_vec = param.as_object()
            .and_then(|i| i.get("evt"))
            .and_then(|i| i.as_array());
        if let Some(data) = evt_vec {
            let mut ble_value = BleValue::default();
            // 数据列表
            for val in data {
                //处理数据
                if let Some(eid) = val.get("eid").and_then(|i| i.as_u64()) {
                    if let Some(edata) = val.get("edata")
                        .and_then(|i| i.as_str())
                        .and_then(|i| <Vec<u8>>::from_hex(i.as_bytes()).ok()) {
                        //tid,edata
                        match Self::mapping_value(eid, edata) {
                            Ok((tp, val)) => {
                                ble_value.set_value(tp, val);
                            }
                            Err(err) => {
                                error!("解析蓝牙事件数据错误错误:{:?}", err);
                            }
                        };
                    }
                }
            }
            return Some(ble_value);
        }
        None
    }
    /// 值映射
    fn mapping_value(eid: u64, edata: Vec<u8>) -> anyhow::Result<(BleValueType, serde_json::Value)> {
        match BleValueType::try_from(eid) {
            Ok(val) => {
                Ok((val, val.unpack(edata)?))
            }
            Err(_) => {
                Err(anyhow!("未知的蓝牙事件: eid:0x{:x},edata:{:?}", eid,edata))
            }
        }
    }
}


#[test]
pub fn test() {
    //8C00
    let token_bytes = <[u8; 2]>::from_hex("D002".as_bytes()).unwrap();
    let a = ValueLsbI16::unpack(&token_bytes).unwrap();
    println!("{:?}", a);
}