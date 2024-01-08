use std::sync::Arc;
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use log::{error, info};
use paho_mqtt::Error::Nul;
use tap::TapFallible;
use tokio::sync::RwLock;
use crate::device::miot_spec_device::{DeviceInfo, MiotSpecDevice};
use crate::device::wifi_device::ExitCode;
use crate::proto::miio_proto::MiIOProtocol;
use crate::proto::transport::gateway_mqtt::OpenMiIOMqttTransport;
use crate::proto::transport::Transport;

/// 连接网关的mqtt

pub struct OpenMiioGatewayDevice {
    info: DeviceInfo,
    proto: Arc<RwLock<Option<Arc<MiIOProtocol>>>>,
    // listener: Arc<RwLock<Vec<i32>>>,
}

impl MiotSpecDevice for OpenMiioGatewayDevice {
    fn get_info(&self) -> &DeviceInfo {
        &self.info
    }
    fn get_proto(&self) -> BoxFuture<Result<Arc<MiIOProtocol>, ExitCode>> {
        async move {
            let read = self.proto.clone();
            let read = read.read().await;
            if let Some(s) = read.clone() {
                return Ok(s);
            }
            drop(read);
            self.connect().await
        }.boxed()
    }

    fn run(&self) -> BoxFuture<Result<(), ExitCode>> {
        async move {
            let p_arc = self.connect().await?;
            p_arc.start_listen().await;
            Err(ExitCode::Disconnect)
        }.boxed()
    }
}

impl OpenMiioGatewayDevice {
    //获取连接
    async fn connect(&self) -> Result<Arc<MiIOProtocol>, ExitCode> {
        let mut write = self.proto.write().await;

        if let Some(s) = write.clone() {
            return Ok(s);
        }

        let ip = self.info.localip.clone().ok_or(ExitCode::ConnectEmpty)?;
        // let wifi_device = WifiDevice::new(info);
        let transport = OpenMiIOMqttTransport::new(ip.as_str(), 1883)
            .await
            .tap_err(|e| error!("连接网关失败,ip:{}: {:?}", ip,e))
            .map_err(|_| ExitCode::ConnectErr)?;
        let proto = MiIOProtocol::new(Transport::OpenMiIOMqtt(transport))
            .await.map_err(|_| ExitCode::ConnectErr)?;
        let p_arc = Arc::new(proto);
        write.replace(p_arc.clone());
        info!("连接网关 mqtt: {} 成功", ip);
        Ok(p_arc)
    }
    pub async fn new(info: DeviceInfo) -> anyhow::Result<Self> {
        let ip = info.localip.clone().expect("网关设备ip不能为空");
        // let wifi_device = WifiDevice::new(info);
        // let transport = OpenMiIOMqttTransport::new(ip.as_str(), 1883).await?;
        // let proto = MiIOProtocol::new(Transport::OpenMiIOMqtt(transport)).await?;


        Ok(Self { info, proto: Arc::new(RwLock::new(None)) })
    }
}