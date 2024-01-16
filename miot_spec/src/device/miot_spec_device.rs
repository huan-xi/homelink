use std::sync::Arc;
use futures_util::future::BoxFuture;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};
use crate::device::ble::ble_device::BleDevice;
use crate::device::emitter::{DataEmitter, DataListener, EventType};
use crate::proto::miio_proto::{MiotSpecDTO, MiotSpecId, MiotSpecProtocolPointer};
use serde_json::Value;
use crate::proto::protocol::{ExitError, JsonMessage};

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
    pub(crate) emitter: Arc<RwLock<DataEmitter<EventType>>>,
    pub tx: broadcast::Sender<EventType>,
}

impl Default for BaseMiotSpecDevice {
    fn default() -> Self {
        let (tx, _) = broadcast::channel(10);
        Self {
            status: RwLock::new(DeviceStatus::Run),
            registered_property: Arc::new(RwLock::new(Vec::new())),
            emitter: Arc::new(RwLock::new(DataEmitter::new())),
            tx,
        }
    }
}

pub enum MiotDeviceType<'a> {
    Wifi,
    Zigbee,
    Ble(&'a BleDevice),
    Mesh,
}

#[async_trait::async_trait]
pub trait MiotSpecDevice {
    fn get_info(&self) -> &DeviceInfo;
    fn get_base(&self) -> &BaseMiotSpecDevice;

    async fn get_proto(&self) -> Result<MiotSpecProtocolPointer, ExitError>;

    /// 设置设备属性
    async fn set_property(&self, siid: i32, piid: i32, value: Value) -> anyhow::Result<()> {
        let did = self.get_info().did.clone();
        let proto = self.get_proto()
            .await
            .map_err(Into::<anyhow::Error>::into)?;
        proto.set_property(MiotSpecDTO { did, siid, piid, value: Some(value) }).await?;
        Ok(())
    }
    /// 读取设备属性
    async fn read_property(&self, siid: i32, piid: i32) -> anyhow::Result<Option<Value>> {
        let did = self.get_info().did.clone();
        let proto = self.get_proto()
            .await
            .map_err(Into::<anyhow::Error>::into)?;
        let value = proto.get_property(MiotSpecDTO { did, siid, piid, value: None }).await?;
        Ok(value)
    }

    async fn get_event_recv(&self) -> broadcast::Receiver<EventType> {
        self.get_base().tx.subscribe()
    }
    /// 创建连接并且 监听
    fn run(&self) -> BoxFuture<Result<(), ExitError>>;

    /// 获取设备类型
    fn get_device_type(&self) -> MiotDeviceType { todo!("无法获取设备类型"); }

    /// 注册属性事件
    async fn register_property(&self, siid: i32, piid: i32) {
        let mut write = self.get_base().registered_property.write().await;
        write.push(MiotSpecId { siid, piid });
    }

    /// 添加指定格式的数据监听器
    async fn emit(&self, event: EventType) {
        self.get_base().emitter.write().await.emit(event).await
    }
    async fn add_listener(&self, listener: DataListener<EventType>) {
        self.get_base().emitter.write().await.add_listener(listener).await
    }
}