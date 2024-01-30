use std::collections::HashMap;
use std::mem;
use std::sync::{Arc};
use std::time::Duration;
use anyhow::anyhow;
use futures_util::future::{BoxFuture, join_all, ok};
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
use crate::device::common::emitter::{DataListener, DataEmitter, EventType,};
use crate::device::MiotDevicePointer;
use crate::proto::miio_proto::{MiotSpecDTO, MiotSpecId, MiotSpecProtocolPointer};
use crate::proto::protocol::{ExitError, RecvMessage};
use crate::proto::transport::ble_mapping_proto::BleMappingProto;

/// https://tasmota.github.io/docs/Bluetooth/
/// 低功耗蓝牙设备
/// 目前低功耗蓝牙设备只支持被动获取数据
///spec_map 用户


pub struct BleDevice {
    pub info: DeviceInfo,
    base: BaseMiotSpecDevice,
    gateway: MiotDevicePointer,
    // 值
    values: Arc<RwLock<BleValue>>,
    spec_map: bimap::BiMap<MiotSpecId, BleValueType>,
    // 传输协议
    proto: Arc<RwLock<Option<MiotSpecProtocolPointer>>>,
}

#[async_trait::async_trait]
impl MiotSpecDevice for BleDevice {
    fn get_info(&self) -> &DeviceInfo { &self.info }

    fn get_base(&self) -> &BaseMiotSpecDevice {
        &self.base
    }

    async fn get_proto(&self) -> Result<MiotSpecProtocolPointer, ExitError> {
        return Err(ExitError::BltConnectErr);
    }


    // read_property
    async fn read_property(&self, siid: i32, piid: i32) -> anyhow::Result<Option<Value>> {
        if let Some(val) = self.spec_map.get_by_left(&MiotSpecId::new(siid, piid)) {
            let read = self.values.read().await;
            if let Some(val) = read.get_value(val.clone()) {
                return Ok(Some(val));
            }
        };
        Ok(None)
    }
    /// 1.同步网关的状态
    async fn run(&self) -> Result<(), ExitError>{
        // let gw_proto = self.gateway.get_proto().await?;
        // let mut recv = gw_proto.recv();
        let mut recv = self.gateway.get_event_recv().await;

        while let Ok(EventType::GatewayMsg(msg)) = recv.recv().await {
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
                            self.set_value_from_param(v).await;
                        }
                    }
                }
            }
        }
        Ok(())
    }
    fn get_device_type(&self) -> MiotDeviceType {
        return MiotDeviceType::Ble(self);
    }
}

impl BleDevice {
    //网关上设置一个监听器,监听属于我的消息
    //蓝牙协议
    pub fn new(info: DeviceInfo, gateway: MiotDevicePointer, spec_map: bimap::BiMap<MiotSpecId, BleValueType>) -> Self {
        Self {
            info,
            base: BaseMiotSpecDevice {
                ..BaseMiotSpecDevice::default()
            },
            gateway,
            spec_map,
            values: Default::default(),
            proto: Arc::new(Default::default()),
        }
    }

    async fn set_value_from_param(&self, param: &Value) {
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
                                error!("解析蓝牙事件数据错误错误:{:?},eid:0x{:x}", err, eid);
                            }
                        };
                    }
                }
            }
            //  数据提交
            self.values.write().await.extend(ble_value.clone());
            for (tp, value) in ble_value.value_map.into_iter() {
                match self.spec_map.get_by_right(&tp) {
                    None => {
                        debug!("emit empty:{:?}", tp);
                    }
                    Some(ps) => {
                        let event = EventType::UpdateProperty(MiotSpecDTO {
                            did: self.info.did.clone(),
                            siid: ps.siid,
                            piid: ps.piid,
                            value: Some(value),
                        });

                        self.emit(event).await;
                    }
                };
            }
        }
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