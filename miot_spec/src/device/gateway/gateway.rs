use std::process::ExitCode;
use std::sync::Arc;
use anyhow::anyhow;
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use log::{error, info};
use tap::TapFallible;
use tokio::select;
use tokio::sync::broadcast::Receiver;
use crate::proto::protocol::{ExitError, JsonMessage};
use tokio::sync::RwLock;
use crate::device::emitter::EventType;
use crate::device::miot_spec_device::{BaseMiotSpecDevice, DeviceInfo, MiotSpecDevice};
use crate::proto::miio_proto::{MiotSpecProtocolPointer, MsgCallback};
use crate::proto::transport::open_miio_mqtt_proto::OpenMiIOMqttSpecProtocol;

/// 连接网关的mqtt

pub struct OpenMiioGatewayDevice {
    info: DeviceInfo,
    proto: Arc<RwLock<Option<MiotSpecProtocolPointer>>>,
    base: BaseMiotSpecDevice,
}

#[async_trait::async_trait]
impl MiotSpecDevice for OpenMiioGatewayDevice {
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

    fn run(&self) -> BoxFuture<Result<(), ExitError>> {
        async move {
            let p_arc = self.connect().await?;
            // self.base.tx.send(EventType::GatewayMsg(self.info.clone().into())
            let listen = p_arc.start_listen();
            let forward = async {
                while let Ok(msg) = p_arc.recv().recv().await {
                    let _ = self.base.tx.send(EventType::GatewayMsg(msg));
                };
            };
            loop {
                select! {
                    _ = listen =>{break;}
                    _ = forward =>{break;}
                }
            }
            // futures_util::join!(listen, a);
            Err(ExitError::Disconnect)
        }.boxed()
    }
}

impl OpenMiioGatewayDevice {
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
    pub async fn new(info: DeviceInfo) -> anyhow::Result<Self> {
        let _ = info.localip.clone()
            .ok_or(anyhow!("网关设备ip不能为空"))?;
        let base: BaseMiotSpecDevice = Default::default();
        Ok(Self { info, proto: Arc::new(RwLock::new(None)), base })
    }
}