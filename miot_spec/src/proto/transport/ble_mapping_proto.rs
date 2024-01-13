use std::collections::HashMap;
use std::time::Duration;
use anyhow::anyhow;
use futures_util::future::err;
use futures_util::SinkExt;
use tokio::sync::broadcast::Receiver;
use crate::device::ble::value_types::BleValueType;
use crate::device::MiotDevicePointer;
use crate::proto::miio_proto::{MiotSpecDTO, MiotSpecId, MiotSpecProtocol, MiotSpecProtocolPointer};
use crate::proto::protocol::JsonMessage;

/// 蓝牙数据属性映射协议
/// 蓝牙设备需要通过网关设备去调用
pub struct BleMappingProto {
    gateway: MiotDevicePointer,
}