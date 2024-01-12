use std::sync::Arc;
use futures_util::future::BoxFuture;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use crate::device::ble::ble_device::BleDevice;
use crate::device::value::{DataListener, ListenDateType};
use crate::device::wifi_device::ExitCode;
use crate::proto::miio_proto::MiIOProtocol;
use futures_util::FutureExt;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, )]
pub struct DeviceInfo {
    pub did: String,
    pub token: String,
    pub model: String,
    pub firmware_revision: Option<String>,
    pub software_revision: Option<String>,
    pub name: String,
    /// mac 地址
    pub mac: Option<String>,
    /// 序列号
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub localip: Option<String>,
}

pub enum DeviceStatus {
    /// 正常
    Run,
    /// 断开连接
    Disconnect,
}

pub struct BaseMiotSpecDevice {
    pub status: RwLock<DeviceStatus>,
}

impl Default for BaseMiotSpecDevice {
    fn default() -> Self {
        Self {
            status: RwLock::new(DeviceStatus::Run)
        }
    }
}

pub enum MiotDeviceType<'a> {
    Wifi,
    Zigbee,
    Ble(&'a BleDevice),
    Mesh,
}


pub trait MiotSpecDevice {
    fn get_info(&self) -> &DeviceInfo;

    fn get_proto(&self) -> BoxFuture<Result<Arc<MiIOProtocol>, ExitCode>>;

    /// 创建连接并且 监听
    fn run(&self) -> BoxFuture<Result<(), ExitCode>>;
    fn get_device_type(&self) -> MiotDeviceType { todo!(); }

    /// 注册属性事件
    fn register_property(&self, siid: i32, piid: i32) -> BoxFuture<()> {
        async move { () }.boxed()
    }
    /// 添加指定格式的数据监听器
    fn add_listener(&self, listener: DataListener<ListenDateType>) -> BoxFuture<()> {
        todo!();
    }
}