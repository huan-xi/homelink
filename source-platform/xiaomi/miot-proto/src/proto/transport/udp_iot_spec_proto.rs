use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use aes::Aes128;
use anyhow::anyhow;
use async_trait::async_trait;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use futures_util::{FutureExt, TryStreamExt};
use log::{debug, error, info};
use serde_json::{Map, Value};
use tap::TapFallible;
use tokio::net::UdpSocket;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::time::timeout;

use crate::proto::miio_proto::MiotSpecProtocol;
use crate::proto::protocol::{JsonMessage, Message, MessageHeader};
use crate::utils::timestamp;

/// udp 传输协议
pub struct UdpMiotSpecProtocol {
    pub socket: Arc<UdpSocket>,
    /// Device ID
    device_id: u32,
    /// 设备时间戳
    stamp: u32,
    /// 开始时间戳
    start_stamp: u64,
    token: [u8; 16],
    msg_sender: Sender<JsonMessage>,
    id: AtomicU64,
    /// Device token
    timeout: Duration,
}

impl UdpMiotSpecProtocol {
    pub async fn new(ip: &str, port: u32, token: [u8; 16], timeout: Duration) -> anyhow::Result<Self> {
        let addr: std::net::SocketAddr = format!("{}:{}", ip, port).parse().unwrap();
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(addr).await?;
        let socket = std::sync::Arc::new(socket);
        let (msg_sender, _) = tokio::sync::broadcast::channel(1024);
        let msg = Self::discover(socket.clone().as_ref(), Duration::from_secs(30)).await?;
        //获取时间戳
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        info!("udp 协议连接成功，ip:{:?},{:?},{:?}", addr,msg.header.stamp,millis);
        let start_stamp = timestamp();
        Ok(Self {
            socket,
            stamp: msg.header.stamp,
            start_stamp,
            device_id: msg.header.device_id,
            token,
            msg_sender,
            id: AtomicU64::new(0),
            timeout,
        })
    }

    pub(crate) async fn build_message(&self, cmd: &str) -> anyhow::Result<Message> {
        let data = if cmd.is_empty() {
            vec![]
        } else {
            Utils::encrypt(&self.token, cmd.as_bytes())
        };
        let diff = timestamp() - self.start_stamp;
        let stamp = self.stamp + diff as u32;
        let msg = Message::build(MessageHeader {
            device_id: self.device_id,
            stamp,
            //先用token 作为checksum
            checksum: self.token,
            ..Default::default()
        }, data);

        Ok(msg)
    }
    /// 握手
    /// 扫描设备
    pub async fn discover(socket: &UdpSocket, timeout: Duration) -> anyhow::Result<Message> {
        let helle_bytes = hex::decode("21310020ffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();
        //todo 广播
        for _ in 0..1 {
            // socket.send_to(helle_bytes.as_slice(), &addr).await?;
            socket.send(helle_bytes.as_slice()).await?;
        }
        let mut buf = [0u8; 1024];
        let result = tokio::time::timeout(timeout, socket.recv_from(&mut buf))
            .await
            .tap_err(|_f| error!("hello 包响应超时"))??;
        let (size, _block_modes) = result;
        let msg = Message::parse(&buf[..size]).unwrap();
        Ok(msg)
    }
}

#[async_trait]
impl MiotSpecProtocol for UdpMiotSpecProtocol {
    fn incr_cmd_id(&self) -> u64 {
        self.id.fetch_add(1, Ordering::SeqCst)
    }

    async fn request<'a>(&'a self, id: u64, cmd: &'a str, timeout_val: Option<Duration>) -> anyhow::Result<JsonMessage> {
        let mut rx = self.msg_sender.subscribe();
        self.send(cmd).await?;
        let t = timeout_val.unwrap_or(self.timeout);
        let res = timeout(t, async move {
            loop {
                // let msg = self.msg_sender.subscribe().recv().await?;
                let msg = rx.recv().await?;
                if let Some(val) = msg.data.get("id") {
                    if val.as_u64() == Some(id) {
                        return Ok(msg);
                    }
                };
            }
        }).await.map_err(|_f| anyhow!("调用接口响应超时,cmd_id:{}",id))?;
        if let Err(e) = &res {
            error!("await_result error:{}", e);
            //将状态改成断开
        };
        res
    }

    async fn send<'a>(&'a self, cmd: &'a str) -> anyhow::Result<()> {
        let msg = self.build_message(cmd).await?;
        let data = msg.pack_to_vec();
        self.socket.send(data.as_slice()).boxed().await?;
        Ok(())
    }

    fn recv(&self) -> Receiver<JsonMessage> {
        self.msg_sender.subscribe()
    }


    async fn await_result(&self, id: u64, timeout_val: Option<Duration>) -> anyhow::Result<JsonMessage> {
        let t = timeout_val.unwrap_or(self.timeout);
        let res = timeout(t, async move {
            loop {
                let msg = self.msg_sender.subscribe().recv().await?;
                if let Some(val) = msg.data.get("id") {
                    if val.as_u64() == Some(id) {
                        return Ok(msg);
                    }
                };
            }
        }).await.map_err(|_f| anyhow!("调用接口响应超时,cmd_id:{}",id))?;
        if let Err(e) = &res {
            error!("await_result error:{}", e);
            //将状态改成断开
        };
        res
    }

    async fn start_listen(&self) {
        let mut buf = [0u8; 65535];
        let sender = self.msg_sender.clone();
        info!("设备id:{},开始监听消息udp",self.device_id);
        while let Ok(bytes) = self.socket.recv(buf.as_mut()).await {
            //等待id=当前id的消息
            let vec = buf.to_vec();
            //解包
            let msg = Message::unpack(&self.token, &vec[..bytes]);
            if !msg.data.is_empty() {
                let str = String::from_utf8(msg.data.clone()).unwrap();
                debug!("收到udp消息:{}", str);
                match serde_json::from_str::<Map<String, Value>>(str.as_str()) {
                    Ok(data) => {
                        let _ = sender.send(JsonMessage {
                            // header: Some(msg.header),
                            data,
                        });
                    }
                    Err(e) => {
                        error!("设备消息解析json错误:{:?}", e);
                    }
                };
            }
        }
        // 设备断开
        error!("udp 协议断开");
    }
}


pub struct Utils;


impl Utils {
    pub fn key_iv(token: &[u8; 16]) -> (Vec<u8>, Vec<u8>) {
        let key = md5::compute(token).to_vec();
        let mut iv_src = key.to_vec();
        iv_src.extend(token);
        let iv = md5::compute(iv_src).to_vec();
        (key, iv)
    }
    /// 解密
    pub fn decrypt(token: &[u8; 16], payload: &[u8]) -> Vec<u8> {
        if payload.is_empty() {
            return vec![];
        };
        let (key, iv) = Self::key_iv(token);
        let cipher = Cbc::<Aes128, Pkcs7>::new_from_slices(&key, &iv).unwrap();
        let mut buf = payload.to_vec();
        cipher.decrypt(&mut buf).unwrap().to_vec()
    }
    /// 加密
    pub fn encrypt(token: &[u8; 16], payload: &[u8]) -> Vec<u8> {
        let (key, iv) = Self::key_iv(token);
        let cipher = Cbc::<Aes128, Pkcs7>::new_from_slices(&key, &iv).unwrap();
        cipher.encrypt_vec(payload)
    }
}
