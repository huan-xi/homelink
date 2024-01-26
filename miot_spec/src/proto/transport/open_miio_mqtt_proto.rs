use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use anyhow::anyhow;
use futures_util::StreamExt;
use log::{debug, error, info};
use paho_mqtt::{AsyncClient, Message};
use serde_json::{Map, Value};
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::RwLock;
use tokio::time::timeout;
use crate::proto::miio_proto::{MiotSpecProtocol, MsgCallback};
use crate::proto::protocol::JsonMessage;
pub use paho_mqtt::Message as MqttMessage;
use paho_mqtt as mqtt;

/// 基于 miio 的网关传输协议
pub struct OpenMiIOMqttSpecProtocol {
    pub client: AsyncClient,
    pub receiver: Arc<RwLock<async_channel::Receiver<Option<Message>>>>,
    msg_sender: Sender<JsonMessage>,
    id: AtomicU64,
    /// Device token
    timeout: Duration,
}

impl OpenMiIOMqttSpecProtocol {
    pub async fn new(ip: &str, port: u32) -> anyhow::Result<Self> {
        let url = format!("mqtt://{}:{}", ip, port);
        let create_opts = mqtt::CreateOptionsBuilder::new_v3()
            .server_uri(url)
            .client_id("home_gateway")
            .finalize();
        let mut client = mqtt::AsyncClient::new(create_opts)?;
        // Create the connect options, explicitly requesting MQTT v3.x
        let conn_opts = mqtt::ConnectOptionsBuilder::new_v3()
            .keep_alive_interval(Duration::from_secs(30))
            .clean_session(false)
            // .will_message(lwt)
            .finalize();
        client.connect(conn_opts).await?;
        client.subscribe("#", 1).await?;
        let (msg_sender, _) = tokio::sync::broadcast::channel(2048);
        let strm = client.get_stream(100);
        //启动消息监听
        Ok(Self {
            client,
            msg_sender,
            id: Default::default(),
            receiver: Arc::new(RwLock::new(strm)),
            timeout: Duration::from_secs(10),
        })
    }
}

#[async_trait::async_trait]
impl MiotSpecProtocol for OpenMiIOMqttSpecProtocol {
    fn incr_cmd_id(&self) -> u64 {
        self.id.fetch_add(1, Ordering::SeqCst)
    }
    async fn send<'a>(&'a self, cmd: &'a str) -> anyhow::Result<()> {
        if !self.client.is_connected() {
            self.client.reconnect().await?;
        };
        let msg = MqttMessage::new("miio/command", cmd, paho_mqtt::QOS_1);
        //发送 "miio/command",
        self.client.publish(msg).await?;
        Ok(())
    }


    fn recv(&self) -> Receiver<JsonMessage> {
        self.msg_sender.subscribe()
    }

    async fn await_result(&self, id: u64, timeout_val: Option<Duration>) -> anyhow::Result<JsonMessage> {
        let tv = timeout_val.unwrap_or(self.timeout);
        let mut recv = self.msg_sender.subscribe();
        timeout(tv, async move {
            loop {
                let msg = recv.recv().await?;
                if let Some(val) = msg.data.get("id") {
                    if val.as_u64() == Some(id as u64) {
                        debug!("await_result:{:?}", msg.data);
                        return Ok(msg);
                    }
                };
            }
        }).await.map_err(|e| anyhow!("执行命令超时"))?
    }

    async fn start_listen(&self) {
        let receiver = self.receiver.clone();
        // let sender = &self.msg_sender;
        let mut write = receiver.write().await;
        while let Some(msg) = write.next().await {
            if let Some(msg) = msg {
                //命令确定
                //"central/report"
                // "miio/command_ack"
                //转到sender
                let str = msg.payload_str().to_string();
                debug!("Received topic:{} message: {}", msg.topic(), msg.payload_str());
                match msg.topic() {
                    "central/report" | "miio/command_ack" => {
                        match serde_json::from_str::<Map<String, Value>>(str.as_str()) {
                            Ok(data) => {
                                let _ = self.msg_sender.send(
                                    JsonMessage {
                                        data,
                                    }
                                );
                            }
                            Err(err) => {
                                error!("解析数据失败: {:?},str:{}", err,str);
                            }
                        }
                    }
                    _ => {}
                };

                // if msg.topic().eq("central/report") || msg.topic().eq("miio/command_ack") {}
            }
        }
        error!("读取数据失败,mqtt 客户端断开");
    }
}