use std::cmp::{max, min};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use hex::FromHex;
use log::{error, info};
use serde::{Deserialize, Serialize};
use tap::TapFallible;
use tokio::select;
use tokio::sync::RwLock;
use hl_integration::JsonValue;

use crate::device::common::utils::get_poll_func;
use crate::device::miot_spec_device::{BaseMiotSpecDevice, DeviceInfo, MiotDeviceType, MiotSpecDevice, MiotSpecDeviceWrapper};
use crate::proto::miio_proto::{MiotSpecId, MiotSpecProtocolPointer};
use crate::proto::protocol::ExitError;
use crate::proto::protocol::ExitError::ConnectErr;
use crate::proto::transport::udp_iot_spec_proto::UdpMiotSpecProtocol;

pub type WifiDevice = MiotSpecDeviceWrapper;

pub struct WifiDeviceInner {
    pub base: BaseMiotSpecDevice,
    pub info: DeviceInfo,
    ///协议
    proto: Arc<RwLock<Option<MiotSpecProtocolPointer>>>,
    /// 轮询间隔
    pub interval: Duration,
    /// 超时时间
    /// udp 协议并发高可能不会返回数据
    timeout: Duration,
}


#[async_trait::async_trait]
impl MiotSpecDevice for WifiDeviceInner {
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
    async fn run(&self) -> Result<(), ExitError> {
        let proto = self.connect().await?;
        //开启状态获取
        let listen = proto.start_listen();
        // 开启轮询
        let poll = get_poll_func(self, self.info.did.as_str(), self.interval);
        loop {
            select! {
                    _ = listen => break,
                    _ = poll => break,
                }
        }
        self.proto.write().await.take();
        Err(ExitError::Disconnect)
        // 该表当前设备的状态
        //ExitCode::OK
    }
}


impl WifiDeviceInner {
    // 设置协议
    async fn connect(&self) -> Result<MiotSpecProtocolPointer, ExitError> {
        let mut write = self.proto.write().await;
        info!("开始连接设备:{}", self.info.did);
        // 避免重复创建
        if let Some(s) = write.clone() {
            return Ok(s);
        }
        let token_bytes = <[u8; 16]>::from_hex(self.info.token.as_bytes())
            .map_err(|_| ExitError::InvalidToken)?;
        //arp 获取map地址? mdns 获取ip
        let ip = self.info.localip.as_ref()
            .ok_or(ExitError::ConnectEmpty)?; // .expect("ip 不能为空");
        let port = 54321;
        let udp = UdpMiotSpecProtocol::new(ip.as_str(), port, token_bytes, self.timeout)
            .await
            .tap_err(|e| error!("udp 连接:{}:{}失败:{}",ip,port ,e))
            .map_err(|_| ConnectErr)?;
        let proto = Arc::new(udp);
        write.replace(proto.clone());
        Ok(proto)
    }
}

#[derive(Serialize, Deserialize)]
pub struct WifiParam {
    pub interval: Option<u64>,
    pub timeout: Option<u64>,
    pub poll_properties: Vec<MiotSpecId>,
}

impl WifiDevice {
    pub fn new_wifi_device(info: DeviceInfo, params: JsonValue) -> anyhow::Result<Self> {
        let wifi_param: WifiParam = serde_json::from_value(params)
            .tap_err(|e| error!("wifi 参数错误:{}", e))?;
        let poll_properties = wifi_param.poll_properties
            .into_iter()
            .collect::<HashSet<MiotSpecId>>();
        let inner = WifiDeviceInner {
            base: BaseMiotSpecDevice {
                poll_properties: Arc::new(RwLock::new(poll_properties)),
                ..std::default::Default::default()
            },
            info,
            proto: Arc::new(RwLock::new(None)),
            interval: Duration::from_secs(wifi_param.interval
                .map(|i| max(i, 1))
                .unwrap_or(120)),

            timeout: Duration::from_millis(wifi_param.timeout.unwrap_or(2_000)),
        };
        Ok(MiotSpecDeviceWrapper(Box::new(inner), MiotDeviceType::Wifi))
    }
}