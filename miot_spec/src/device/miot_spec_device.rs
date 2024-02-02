use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use anyhow::anyhow;
use futures_util::future::BoxFuture;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, Mutex, RwLock};
use crate::device::ble::ble_device::BleDevice;
use crate::device::common::emitter::{DataEmitter, DataListener, EventType};
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
    pub manufacturer: Option<String>,
    pub localip: Option<String>,
    pub extra: Option<Extra>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, )]
pub struct Extra {
    pub fw_version: Option<String>,
}


pub enum DeviceStatus {
    /// 正常
    Run,
    /// 断开连接
    Disconnect,
}

pub struct RetryInfo {
    /// 重试次数
    pub retry_count: Mutex<u32>,
    /// 最大重试间隔 5 分钟,单位毫秒
    pub max_interval: u32,
}

impl RetryInfo {
    pub async fn incr(&self) -> u32 {
        let mut write = self.retry_count.lock().await;
        *write += 1;
        return *write;
    }
    pub async fn reset(&self) {
        let mut write = self.retry_count.lock().await;
        *write = 0;
    }
    pub async fn get(&self) -> u32 {
        let read = self.retry_count.lock().await;
        // 产生1-1000 随机数
        let rand = rand::random::<u32>() % 1000 + 1;
        2u32.pow(*read - 1) * 1000 + rand
    }
}

#[tokio::test]
async fn test_retry_info() {
    let retry_info = RetryInfo::default();
    for i in 0..10 {
        retry_info.incr().await;
        let info = retry_info.get().await;
        println!("{}:{}", i, info);
    }
}

impl Default for RetryInfo {
    fn default() -> Self {
        Self {
            retry_count: Mutex::new(0),
            max_interval: 60_1000,
        }
    }
}


pub struct BaseMiotSpecDevice {
    pub status: RwLock<DeviceStatus>,
    /// 注册轮询的属性
    pub poll_properties: Arc<RwLock<HashSet<MiotSpecId>>>,
    /// 存储属性数据
    pub value_map: Arc<RwLock<HashMap<MiotSpecId, serde_json::Value>>>,
    pub(crate) emitter: Arc<RwLock<DataEmitter<EventType>>>,
    pub tx: broadcast::Sender<EventType>,
    pub retry_info: RetryInfo,
}

impl Default for BaseMiotSpecDevice {
    fn default() -> Self {
        let (tx, _) = broadcast::channel(10);
        Self {
            status: RwLock::new(DeviceStatus::Run),
            poll_properties: Arc::new(RwLock::new(HashSet::new())),
            value_map: Arc::new(Default::default()),
            emitter: Arc::new(RwLock::new(DataEmitter::new())),
            tx,
            retry_info: Default::default(),
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

    /// 设置设备属性 siid: i32, piid: i32
    async fn set_property(&self, spec_id: MiotSpecId, value: Value) -> anyhow::Result<()> {
        let did = self.get_info().did.clone();
        let proto = self.get_proto()
            .await
            .map_err(Into::<anyhow::Error>::into)?;
        proto.set_property(MiotSpecDTO { did, siid: spec_id.siid, piid: spec_id.piid, value: Some(value) }).await?;
        Ok(())
    }
    /// 读取设备属性
    async fn read_property(&self, siid: i32, piid: i32) -> anyhow::Result<Option<Value>> {
        let did = self.get_info().did.clone();
        let proto = self.get_proto()
            .await
            .map_err(Into::<anyhow::Error>::into)?;
        let value = proto.get_property(MiotSpecDTO { did, siid, piid, value: None }).await?;
        self.get_base().retry_info.reset().await;
        Ok(value)
    }

    async fn read_properties(&self, props: Vec<MiotSpecId>) -> anyhow::Result<Vec<MiotSpecDTO>> {
        let did = self.get_info().did.clone();
        let props: Vec<MiotSpecDTO> = props.into_iter()
            .map(|id| MiotSpecDTO { did: did.clone(), siid: id.siid, piid: id.piid, value: None }).collect();
        let len = props.len();
        let proto = self.get_proto()
            .await
            .map_err(Into::<anyhow::Error>::into)?;
        let value = proto.get_properties(props, None).await?;
        if value.len() != len {
            return Err(anyhow!("读取属性失败,返回值数量不匹配"));
        }
        self.get_base().retry_info.reset().await;
        Ok(value)
    }

    async fn get_event_recv(&self) -> broadcast::Receiver<EventType> {
        self.get_base().tx.subscribe()
    }
    /// 创建连接并且 监听
    async fn run(&self) -> Result<(), ExitError>;

    /// 获取设备类型
    fn get_device_type(&self) -> MiotDeviceType { todo!("无法获取设备类型"); }

    /// 注册属性事件
    async fn register_property(&self, siid: i32, piid: i32) {
        let mut write = self.get_base().poll_properties.write().await;
        write.insert(MiotSpecId { siid, piid });
    }

    /// 添加指定格式的数据监听器
    async fn emit(&self, event: EventType) {
        self.get_base().emitter.write().await.emit(event).await
    }
    async fn add_listener(&self, listener: DataListener<EventType>) {
        self.get_base().emitter.write().await.add_listener(listener)
    }
}