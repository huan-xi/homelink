use std::sync::{Arc};
use std::time::Duration;
use anyhow::{anyhow, Error};
use futures_util::future::{BoxFuture, join};
use hex::FromHex;
use crate::device::miot_spec_device::{BaseMiotSpecDevice, DeviceInfo, MiotSpecDevice};
use crate::proto::miio_proto::{MiIOProtocol, MiotSpecDTO, MiotSpecId};
use crate::proto::transport::miio_udp_transport::UdpTransport;
use crate::proto::transport::Transport;
use futures_util::FutureExt;
use log::{error, info};
use tap::TapFallible;
use tokio::sync::RwLock;
use crate::device::value::{DataEmitter, DataListener, ListenDateType};
use crate::device::wifi_device::ExitCode::ConnectErr;

pub struct WifiDevice {
    pub base: BaseMiotSpecDevice,
    pub info: DeviceInfo,
    ///协议
    proto: Arc<RwLock<Option<Arc<MiIOProtocol>>>>,
    /// 轮询间隔
    pub interval: Duration,
    /// 注册轮询的属性
    pub registered_property: Arc<RwLock<Vec<MiotSpecId>>>,
    emitter: Arc<RwLock<DataEmitter<ListenDateType>>>,
}

#[derive(Debug)]
pub enum ExitCode {
    /// 连接信息为空
    ConnectEmpty,
    /// token 非法
    InvalidToken,

    Disconnect,
    ConnectErr,
    Lock,
}

impl Into<anyhow::Error> for ExitCode {
    fn into(self) -> Error {
        anyhow::anyhow!("{:?}",self)
    }
}


impl MiotSpecDevice for WifiDevice {
    fn get_info(&self) -> &DeviceInfo { &self.info }


    fn get_proto(&self) -> BoxFuture<Result<Arc<MiIOProtocol>, ExitCode>> {
        async move {
            // 等待设备运行
            let read = self.proto.clone();
            let read = read.read().await;
            if let Some(s) = read.clone() {
                return Ok(s);
            }
            drop(read);
            self.connect().await
        }.boxed()
    }
    /// 连接协议,并且监听
    fn run(&self) -> BoxFuture<Result<(), ExitCode>> {
        async move {
            let proto = self.connect().await?;
            //开启状态获取
            let listen = proto.start_listen();
            // 开启轮询
            let poll = async {
                loop {
                    tokio::time::sleep(self.interval).await;
                    if self.emitter.read().await.is_empty() {
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

                    let params = self.registered_property
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
                            self.emitter.clone().write().await.emit(ListenDateType::MiotProp(result)).await;
                        }
                    }
                }
            };
            join(listen, poll).await;
            Err(ExitCode::Disconnect)
            // 该表当前设备的状态
            //ExitCode::OK
        }.boxed()
    }
    fn register_property(&self, siid: i32, piid: i32) -> BoxFuture<()> {
        async move {
            let mut write = self.registered_property.write().await;
            write.push(MiotSpecId { siid, piid });
        }.boxed()
    }
    fn add_listener(&self, listener: DataListener<ListenDateType>) -> BoxFuture<()> {
        async move {
            self.emitter.clone().write().await.add_listener(listener).await;
        }.boxed()
    }
}

impl WifiDevice {
    /// 设置协议
    async fn connect(&self) -> Result<Arc<MiIOProtocol>, ExitCode> {
        let mut write = self.proto.write().await;
        // 避免重复创建
        if let Some(s) = write.clone() {
            return Ok(s);
        }
        let token_bytes = <[u8; 16]>::from_hex(self.info.token.as_bytes())
            .map_err(|_| ExitCode::InvalidToken)?;
        //arp 获取map地址? mdns 获取ip
        let ip = self.info.localip.as_ref()
            .ok_or(ExitCode::ConnectEmpty)?; // .expect("ip 不能为空");
        let udp = UdpTransport::new(ip.as_str(), 54321, token_bytes)
            .await
            .tap_err(|e| error!("udp 连接失败:{:?}", e))
            .map_err(|_| ConnectErr)?;
        let proto = MiIOProtocol::new(Transport::Udp(udp)).await.map_err(|_| ConnectErr)?;
        let arc = Arc::new(proto);
        write.replace(arc.clone());
        Ok(arc)
    }
    pub async fn new(info: DeviceInfo) -> anyhow::Result<Self> {
        Ok(WifiDevice {
            base: BaseMiotSpecDevice {
                ..std::default::Default::default()
            },
            info,
            proto: Arc::new(RwLock::new(None)),
            interval: Duration::from_secs(70),
            registered_property: Arc::new(RwLock::new(Vec::new())),
            emitter: Arc::new(RwLock::new(DataEmitter::new())),
        })
    }
}