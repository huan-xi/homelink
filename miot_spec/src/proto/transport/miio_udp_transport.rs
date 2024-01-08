use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use log::{error, info};
use serde_json::Value;
use tokio::net::UdpSocket;
use tokio::sync::broadcast::Sender;
use tokio::sync::RwLock;
use tokio::time::timeout;
use crate::proto::protocol;
use crate::proto::protocol::{JsonMessage, Message, MessageHeader};
use crate::proto::transport::MiioTransport;
use crate::utils::timestamp;


/// udp 传输协议
pub struct UdpTransport {
    pub socket: Arc<UdpSocket>,
    /// Device ID
    device_id: u32,
    stamp: u32,
    /// 开始时间戳
    start_stamp: u64,
    token: [u8; 16],
    connect: AtomicBool,
    pub msg_sender: Sender<JsonMessage>,
}

impl UdpTransport {
    pub async fn new(ip: &str, port: u32, token: [u8; 16]) -> anyhow::Result<Self> {
        let addr: std::net::SocketAddr = format!("{}:{}", ip, port).parse().unwrap();
        let mut socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(addr).await?;
        let socket = std::sync::Arc::new(socket);
        let (msg_sender, _) = tokio::sync::broadcast::channel(10);
        let msg = Self::discover(socket.clone().as_ref(), Duration::from_secs(30)).await?;
        //获取时间戳
        // let now = Utc::now();
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
            connect: AtomicBool::new(true),
            msg_sender,
        })
    }

    pub(crate) async fn build_message(&self, id: u32, cmd: &str) -> anyhow::Result<Message> {
        let data = if cmd.is_empty() {
            vec![]
        } else {
            protocol::Utils::encrypt(&self.token, cmd.as_bytes())
        };
        let diff = timestamp() - self.start_stamp;
        let stamp = self.stamp + diff as u32;
        let msg = Message::build(MessageHeader {
            device_id: self.device_id,
            stamp,
            //先用token 作为checksum
            checksum: self.token.clone(),
            ..Default::default()
        }, data);

        Ok(msg)
    }
    /// 扫描设备
    pub async fn discover(socket: &UdpSocket, timeout: Duration) -> anyhow::Result<Message> {
        let helle_bytes = hex::decode("21310020ffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();
        //todo 广播
        for _ in 0..1 {
            // socket.send_to(helle_bytes.as_slice(), &addr).await?;
            socket.send(helle_bytes.as_slice()).await?;
        }
        let mut buf = [0u8; 1024];
        let result = tokio::time::timeout(timeout, socket.recv_from(&mut buf)).await??;
        let (size, block_modes) = result;
        let msg = Message::parse(&buf[..size]).unwrap();
        Ok(msg)
    }

    pub async fn await_result(&self, id: u32, tv: Duration) -> anyhow::Result<JsonMessage> {
        let res = timeout(tv, async move {
            loop {
                let msg = self.recv().recv().await?;
                if let Some(val) = msg.data.get("id") {
                    if val.as_u64() == Some(id as u64) {
                        return Ok(msg);
                    }
                };
            }
        }).await?;
        if let Err(e) = &res {
            error!("await_result error:{:?}", e);
            //将状态改成断开
            self.connect.store(false, Ordering::SeqCst);
        };
        res
    }
}

impl MiioTransport<Vec<u8>, JsonMessage> for UdpTransport {
    fn send(&self, data: Vec<u8>) -> BoxFuture<anyhow::Result<()>> {
        async move {
            self.socket.send(data.as_slice()).boxed().await?;
            Ok(())
        }.boxed()
    }

    /// 返回的是一个消息流
    fn recv(&self) -> tokio::sync::broadcast::Receiver<JsonMessage> {
        let mut receiver = self.msg_sender.subscribe();
        receiver
    }

    /// 开始监听消息
    fn start_listen(&self) -> BoxFuture<()> {
        async move {
            let mut buf = [0u8; 65535];
            let sender = self.msg_sender.clone();
            info!("设备id:{},开始监听消息udp",self.device_id);
            while let Ok(bytes) = self.socket.recv(buf.as_mut()).await {
                //等待id=当前id的消息
                let vec = buf.to_vec();
                //解包
                let msg = Message::unpack(&self.token, &vec[..bytes]);
                let str = String::from_utf8(msg.data.clone()).unwrap();
                info!("收到udp消息:{}", str);
                serde_json::from_str::<Value>(str.as_str()).unwrap();
                let _ = sender.send(JsonMessage {
                    // header: Some(msg.header),
                    data: serde_json::from_str(str.as_str()).unwrap(),
                });
            }
            // 设备断开
            error!("udp 协议断开");
        }.boxed()
    }
}
