use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{broadcast, RwLock};
use hl_device::error::DeviceExitError;

use hl_device::hl_device::RetryInfo;
use hl_device::{CharReadParam, CharUpdateParam, HlDevice, ReadCharResults, UpdateCharResults};
use hl_device::event::HlDeviceListenable;
use hl_device::platform::hap::hap_device_ext::{AsHapDeviceExt, HapDeviceExt};

use crate::device::common::emitter::{DataEmitter, DataListener, EventType};
use crate::device::common::utils::get_hap_device_info;
use crate::proto::miio_proto::{MiotSpecDTO, MiotSpecId, MiotSpecProtocolPointer};
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


#[tokio::test]
async fn test_retry_info() {
    let retry_info = RetryInfo::default();
    for i in 0..10 {
        retry_info.incr().await;
        let info = retry_info.get().await;
        println!("{}:{}", i, info);
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

pub enum MiotDeviceType {
    Wifi,
    Zigbee,
    Ble,
    Mesh,
}

#[async_trait::async_trait]
pub trait MiotSpecDevice: Sync + Send {
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
    async fn set_properties(&self, params: Vec<(MiotSpecId, Value)>) -> anyhow::Result<Vec<MiotSpecDTO>> {
        let did = self.get_info().did.clone();
        let proto = self.get_proto()
            .await
            .map_err(Into::<anyhow::Error>::into)?;
        let params = params.into_iter().map(|id| MiotSpecDTO { did: did.clone(), siid: id.0.siid, piid: id.0.piid, value: Some(id.1) }).collect();
        proto.set_properties(params, None).await
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


pub struct MiotSpecDeviceWrapper<T: MiotSpecDevice>(pub(crate) T);

#[async_trait::async_trait]
impl<T: MiotSpecDevice> HlDevice for MiotSpecDeviceWrapper<T> {
    fn dev_id(&self) -> &str {
        self.0.get_info().did.as_str()
    }

    async fn run(&self) -> Result<(), Box<dyn DeviceExitError>> {
        self.0.run().await.map_err(|e| Box::new(e) as Box<dyn DeviceExitError>)
    }

    fn retry_info(&self) -> &RetryInfo {
        &self.0.get_base().retry_info
    }
}

impl<T: MiotSpecDevice> HlDeviceListenable for MiotSpecDeviceWrapper<T> {}

#[async_trait::async_trait]
impl<T: MiotSpecDevice> HapDeviceExt for MiotSpecDeviceWrapper<T> {
    fn get_hap_info(&self) -> hl_device::platform::hap::hap_device_ext::DeviceInfo {
        get_hap_device_info(&self.0)
    }

    async fn read_chars_value(&self, params: Vec<CharReadParam>) -> ReadCharResults {
        todo!()
    }

    async fn update_chars_value(&self, params: Vec<CharUpdateParam>) -> UpdateCharResults {
        todo!()
    }
}


impl<T: MiotSpecDevice> AsMiotDevice for MiotSpecDeviceWrapper<T> {
    fn as_miot_device(&self) -> Result<&dyn MiotSpecDevice, NotSupportMiotDeviceError> {
        Ok(&self.0)
    }
}

pub struct NotSupportMiotDeviceError;

impl From<NotSupportMiotDeviceError> for anyhow::Error {
    fn from(value: NotSupportMiotDeviceError) -> Self {
        anyhow!("该设备不是米家设备类型")
    }
}



pub trait AsMiotDevice: Sync + Send {
    fn as_miot_device(&self) -> Result<&dyn MiotSpecDevice, NotSupportMiotDeviceError> {
        Err(NotSupportMiotDeviceError)
    }
}

pub trait AsMiotGatewayDevice: Sync + Send {
    fn as_miot_gateway_device(&self) -> Option<&dyn MiotSpecDevice> {
        None
    }
}

