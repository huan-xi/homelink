use std::sync::Arc;
use futures_util::future::BoxFuture;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use crate::device::ble::ble_device::BleDevice;
use crate::device::emitter::{DataEmitter, DataListener, ListenDateType};
use crate::proto::miio_proto::{MiotSpecId, MiotSpecProtocolPointer};
use futures_util::FutureExt;
use crate::proto::protocol::ExitError;

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
    /// 注册轮询的属性
    pub registered_property: Arc<RwLock<Vec<MiotSpecId>>>,
    pub(crate) emitter: Arc<RwLock<DataEmitter<ListenDateType>>>,

}

impl Default for BaseMiotSpecDevice {
    fn default() -> Self {
        Self {
            status: RwLock::new(DeviceStatus::Run),
            registered_property: Arc::new(RwLock::new(Vec::new())),
            emitter: Arc::new(RwLock::new(DataEmitter::new())),
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
    fn get_base(&self) -> &BaseMiotSpecDevice;

    fn get_proto(&self) -> BoxFuture<Result<MiotSpecProtocolPointer, ExitError>>;

    /// 创建连接并且 监听
    fn run(&self) -> BoxFuture<Result<(), ExitError>>;
    fn get_device_type(&self) -> MiotDeviceType { todo!(); }

    /// 注册属性事件
    fn register_property(&self, siid: i32, piid: i32) -> BoxFuture<()> where
        Self:  Send + Sync {
        async move {
            let mut write = self.get_base().registered_property.write().await;
            write.push(MiotSpecId { siid, piid });
        }.boxed()
    }
    /// 添加指定格式的数据监听器
    fn emit(&self, event: ListenDateType) -> BoxFuture<()> where Self: Sync{
        async move {
            self.get_base().emitter.write()
                .await.emit(event).await;
        }.boxed()
    }
    fn add_listener(&self, listener: DataListener<ListenDateType>) -> BoxFuture<()> where Self: Sync{
        async move {
            self.get_base().emitter.write().await.add_listener(listener).await;
        }.boxed()
    }
}