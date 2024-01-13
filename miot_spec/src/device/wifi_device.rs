use std::sync::{Arc};
use std::time::Duration;
use futures_util::future::{BoxFuture, join};
use hex::FromHex;
use crate::device::miot_spec_device::{BaseMiotSpecDevice, DeviceInfo, MiotSpecDevice};
use crate::proto::miio_proto::{MiotSpecProtocolPointer, MiotSpecDTO, MiotSpecId};
use futures_util::FutureExt;
use log::{error, info};
use tap::TapFallible;
use tokio::sync::RwLock;
use crate::device::emitter::{DataEmitter, DataListener, EventType};
use crate::proto::protocol::ExitError;
use crate::proto::protocol::ExitError::ConnectErr;
use crate::proto::transport::udp_iot_spec_proto::UdpMiotSpecProtocol;

pub struct WifiDevice {
    pub base: BaseMiotSpecDevice,
    pub info: DeviceInfo,
    ///协议
    proto: Arc<RwLock<Option<MiotSpecProtocolPointer>>>,
    /// 轮询间隔
    pub interval: Duration,

}

#[async_trait::async_trait]
impl MiotSpecDevice for WifiDevice {
    fn get_info(&self) -> &DeviceInfo { &self.info }
    fn get_base(&self) -> &BaseMiotSpecDevice {
        &self.base
    }

    async fn get_proto(&self) -> Result<MiotSpecProtocolPointer, ExitError> {
        // 等待设备运行
        let read = self.proto.clone();
        let read = read.read().await;
        if let Some(s) = read.clone() {
            return Ok(s);
        }
        drop(read);
        self.connect().await
    }
    /// 连接协议,并且监听
    fn run(&self) -> BoxFuture<Result<(), ExitError>> {
        async move {
            let proto = self.connect().await?;
            //开启状态获取
            let listen = proto.start_listen();
            // 开启轮询
            let poll = async {
                loop {
                    tokio::time::sleep(self.interval).await;
                    if self.base.emitter.read().await.is_empty() {
                        continue;
                    };

                    let proto = match self.get_proto().await {
                        Ok(p) => {
                            p
                        }
                        Err(e) => {
                            error!("获取协议失败");
                            break;
                        }
                    };

                    let params = self.base.registered_property
                        .read()
                        .await.iter()
                        .map(|id| MiotSpecDTO {
                            did: self.info.did.clone(),
                            siid: id.siid,
                            piid: id.piid,
                            value: None,
                        }).collect::<Vec<MiotSpecDTO>>();
                    if params.is_empty() {
                        continue;
                    }
                    if let Ok(results) = proto.get_properties(params, None).await {
                        for result in results {
                            self.base.emitter.clone().write().await.emit(EventType::SetProperty(result)).await;
                        }
                    }
                }
            };
            join(listen, poll).await;
            Err(ExitError::Disconnect)
            // 该表当前设备的状态
            //ExitCode::OK
        }.boxed()
    }
}


impl WifiDevice {
    // 设置协议
    async fn connect(&self) -> Result<MiotSpecProtocolPointer, ExitError> {
        let mut write = self.proto.write().await;
        // 避免重复创建
        if let Some(s) = write.clone() {
            return Ok(s);
        }
        let token_bytes = <[u8; 16]>::from_hex(self.info.token.as_bytes())
            .map_err(|_| ExitError::InvalidToken)?;
        //arp 获取map地址? mdns 获取ip
        let ip = self.info.localip.as_ref()
            .ok_or(ExitError::ConnectEmpty)?; // .expect("ip 不能为空");
        let udp = UdpMiotSpecProtocol::new(ip.as_str(), 54321, token_bytes)
            .await
            .tap_err(|e| error!("udp 连接失败:{:?}", e))
            .map_err(|_| ConnectErr)?;
        let proto = Arc::new(udp);
        write.replace(proto.clone());
        Ok(proto)
    }
    pub async fn new(info: DeviceInfo) -> anyhow::Result<Self> {
        Ok(WifiDevice {
            base: BaseMiotSpecDevice {
                ..std::default::Default::default()
            },
            info,
            proto: Arc::new(RwLock::new(None)),
            interval: Duration::from_secs(70),
        })
    }
}