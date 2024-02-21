use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum_macros::EnumString;
use tokio::sync::{broadcast, RwLock};
use hl_integration::error::DeviceExitError;

use hl_integration::hl_device::{HlDevice, RetryInfo};
// use hl_integration::{CharReadParam, CharUpdateParam, HlDevice, ReadCharResults, UpdateCharResults};
use hl_integration::event::{EventListener, HlDeviceListenable};
use hl_integration::event::emitter::DeviceEventEmitter;
use hl_integration::HlSourceDevice;
use hl_integration::platform::hap::hap_device;
use hl_integration::platform::hap::hap_device::HapDevice;
// use hl_integration::platform::hap::hap_device_ext::{AsHapDeviceExt, HapDeviceExt};

use crate::device::common::emitter::{DataEmitter, DataListener, MijiaEvent};
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

    pub tx: broadcast::Sender<MijiaEvent>,
    // Arc<RwLock<DataEmitter<EventType>>>
    pub(crate) emitter: DeviceEventEmitter,
    pub retry_info: RetryInfo,
}

impl Default for BaseMiotSpecDevice {
    fn default() -> Self {
        let (tx, _) = broadcast::channel(10);
        Self {
            status: RwLock::new(DeviceStatus::Run),
            poll_properties: Arc::new(RwLock::new(HashSet::new())),
            value_map: Arc::new(Default::default()),
            emitter: DeviceEventEmitter::default(),
            tx,
            retry_info: Default::default(),
        }
    }
}

#[derive(strum_macros::AsRefStr, EnumString, Debug, Clone, Copy)]
pub enum MiotDeviceType {
    #[strum(serialize = "xiaomi_wifi")]
    Wifi,
    #[strum(serialize = "xiaomi_gw_zigbee")]
    Zigbee,
    #[strum(serialize = "xiaomi_gw_mqtt")]
    MqttGateway,
    #[strum(serialize = "xiaomi_gw_ble")]
    Ble,
    #[strum(serialize = "xiaomi_cloud")]
    Cloud,
    #[strum(serialize = "xiaomi_gw_mesh")]
    Mesh,
}

#[cfg(test)]
mod test {
    use std::str::FromStr;
    use crate::device::miot_spec_device::MiotDeviceType;

    #[test]
    pub fn test_strum() {
        let s = MiotDeviceType::Ble;
        let a = s.as_ref();
        println!("{}", a);
        // "xiaomi_gw_ble"
        // MiotDeviceType::from_str()
        let b: MiotDeviceType = FromStr::from_str("xiaomi_gw_ble").unwrap();
        println!("{:?}", b);
    }
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

    async fn get_event_recv(&self) -> broadcast::Receiver<MijiaEvent> {
        self.get_base().tx.subscribe()
    }
    /// 创建连接并且 监听
    async fn run(&self) -> Result<(), ExitError>;


    /// 注册属性事件
    async fn register_property(&self, siid: i32, piid: i32) {
        let mut write = self.get_base().poll_properties.write().await;
        write.insert(MiotSpecId { siid, piid });
    }

    fn get_emitter(&self) -> &DeviceEventEmitter {
        &self.get_base().emitter
    }

    // 添加指定格式的数据监听器
    // async fn emit(&self, event: EventType) {
    //     self.get_base().emitter.write().await.emit(event).await
    // }
    // async fn add_listener(&self, listener: DataListener<EventType>) {
    //     self.get_base().emitter.write().await.add_listener(listener)
    // }
}


#[async_trait::async_trait]
impl HlDeviceListenable for dyn MiotSpecDevice {
    async fn add_listener(&self, listener: EventListener) -> i64 {
        self.get_emitter().add_listener(listener).await
    }

    fn remove_listener(&self, id: i64) -> i64 {
        self.get_emitter().remove_listener(id)
    }
}

pub struct MiotSpecDeviceWrapper(pub(crate) Box<dyn MiotSpecDevice>, pub(crate) MiotDeviceType);

impl HlSourceDevice for MiotSpecDeviceWrapper {}

impl AsMiotDevice for MiotSpecDeviceWrapper {
    fn as_miot_device(&self) -> Result<&dyn MiotSpecDevice, NotSupportMiotDeviceError> {
        Ok(self.0.as_ref())
    }
}

#[async_trait::async_trait]
impl HlDevice for MiotSpecDeviceWrapper {
    fn dev_id(&self) -> String {
        self.0.get_info().did.clone()
    }

    fn device_type(&self) -> &str {
        self.1.as_ref()
    }

    async fn run(&self) -> Result<(), Box<dyn DeviceExitError>> {
        self.0.run().await.map_err(|e| Box::new(e) as Box<dyn DeviceExitError>)
    }

    fn retry_info(&self) -> &RetryInfo {
        &self.0.get_base().retry_info
    }
}
#[async_trait::async_trait]
impl HlDeviceListenable for MiotSpecDeviceWrapper {
    async fn add_listener(&self, listener: EventListener) -> i64 {
        self.0.add_listener(listener).await
    }

    fn remove_listener(&self, id: i64) -> i64 {
        self.0.remove_listener(id)
    }
}

#[async_trait::async_trait]
impl HapDevice for MiotSpecDeviceWrapper {
    fn get_hap_info(&self) -> hap_device::DeviceInfo {
        get_hap_device_info(self.0.as_ref().get_info())
    }
}


pub struct NotSupportMiotDeviceError;

impl Display for NotSupportMiotDeviceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("该设备不是米家设备类型")
    }
}

impl Debug for NotSupportMiotDeviceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("该设备不是米家设备类型")
    }
}

impl From<NotSupportMiotDeviceError> for anyhow::Error {
    fn from(value: NotSupportMiotDeviceError) -> Self {
        anyhow!("{:?}",value)
    }
}


pub trait AsMiotDevice: Sync + Send {
    fn as_miot_device(&self) -> Result<&dyn MiotSpecDevice, NotSupportMiotDeviceError> {
        Err(NotSupportMiotDeviceError)
    }
}

/// HlSourceDevice wrapper
#[derive(Clone)]
pub struct MiotDeviceArc(pub Arc<dyn HlSourceDevice>);

impl MiotDeviceArc {
    pub async fn read_property(&self, siid: i32, piid: i32) -> anyhow::Result<Option<Value>> {
        self.as_miot_device()?.read_property(siid, piid).await
    }
    pub async fn set_property(&self, spec_id: MiotSpecId, value: Value) -> anyhow::Result<()> {
        self.as_miot_device()?.set_property(spec_id, value).await
    }
}

pub struct MiotDeviceBox {
    pub dev: Box<dyn MiotSpecDevice>,
}

impl AsMiotDevice for MiotDeviceArc {
    fn as_miot_device(&self) -> Result<&dyn MiotSpecDevice, NotSupportMiotDeviceError> {
        self.0.as_ref()
            .downcast_ref::<MiotSpecDeviceWrapper>()
            .ok_or(NotSupportMiotDeviceError)?
            .as_miot_device()
    }
}


