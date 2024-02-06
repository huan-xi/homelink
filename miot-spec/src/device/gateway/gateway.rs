use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use anyhow::anyhow;


use log::{error, info};
use tap::TapFallible;
use tokio::select;

use crate::proto::protocol::{ExitError};
use tokio::sync::RwLock;
use hl_device::event::HlDeviceListenable;
use hl_device::HlDevice;
use crate::device::common::emitter::EventType;
use crate::device::miot_spec_device::{AsMiotGatewayDevice, BaseMiotSpecDevice, DeviceInfo, MiotSpecDevice, MiotSpecDeviceWrapper};
use crate::device::common::utils::get_poll_func;
use crate::proto::miio_proto::{MiotSpecProtocolPointer};
use crate::proto::transport::open_miio_mqtt_proto::OpenMiIOMqttSpecProtocol;

pub type OpenMiioGatewayDevice = MiotSpecDeviceWrapper<OpenMiioGatewayDeviceInner>;

impl Deref for OpenMiioGatewayDevice {
    type Target = OpenMiioGatewayDeviceInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


/// 连接网关的mqtt

pub struct OpenMiioGatewayDeviceInner {
    info: DeviceInfo,
    proto: Arc<RwLock<Option<MiotSpecProtocolPointer>>>,
    base: BaseMiotSpecDevice,
    /// 轮询间隔
    pub interval: Duration,
}


#[async_trait::async_trait]
impl MiotSpecDevice for OpenMiioGatewayDeviceInner {
    fn get_info(&self) -> &DeviceInfo {
        &self.info
    }

    fn get_base(&self) -> &BaseMiotSpecDevice {
        &self.base
    }
    async fn get_proto(&self) -> Result<MiotSpecProtocolPointer, ExitError> {
        let read = self.proto.clone();
        let read = read.read().await;
        if let Some(s) = read.clone() {
            return Ok(s);
        }
        drop(read);
        self.connect().await
    }

    async fn run(&self) -> Result<(), ExitError> {
        let p_arc = self.connect().await?;
        // self.base.tx.send(EventType::GatewayMsg(self.info.clone().into())
        let listen = p_arc.start_listen();
        let forward = async {
            while let Ok(msg) = p_arc.recv().recv().await {
                let _ = self.base.tx.send(EventType::GatewayMsg(msg));
            };
        };

        let poll = get_poll_func(self, self.info.did.as_str(), self.interval);
        loop {
            select! {
                    _ = listen =>break,
                    _ = forward =>break,
                    _ = poll => break
                }
        }
        // futures_util::join!(listen, a);
        Err(ExitError::Disconnect)
    }
}

impl OpenMiioGatewayDeviceInner {
    //获取连接
    async fn connect(&self) -> Result<MiotSpecProtocolPointer, ExitError> {
        let mut write = self.proto.write().await;
        if let Some(s) = write.clone() {
            return Ok(s);
        }
        let ip = self.info.localip.clone().ok_or(ExitError::ConnectEmpty)?;
        let proto = OpenMiIOMqttSpecProtocol::new(ip.as_str(), 1883)
            .await
            .tap_err(|e| error!("连接网关失败,ip:{}: {}", ip,e))
            .map_err(|_| ExitError::ConnectErr)?;
        let p_arc = Arc::new(proto);
        write.replace(p_arc.clone());
        info!("连接网关 mqtt: {} 成功", ip);
        Ok(p_arc)
    }
}

impl OpenMiioGatewayDevice {
    pub async fn new(info: DeviceInfo) -> anyhow::Result<Self> {
        let _ = info.localip.clone()
            .ok_or(anyhow!("网关设备ip不能为空"))?;
        let base: BaseMiotSpecDevice = Default::default();
        Ok(MiotSpecDeviceWrapper(OpenMiioGatewayDeviceInner { info, proto: Arc::new(RwLock::new(None)), base, interval: Duration::from_secs(1) }))
    }
}