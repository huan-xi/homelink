use std::sync::{Arc};
use std::time::Duration;
use futures_util::future::BoxFuture;
use futures_util::{FutureExt, StreamExt};
use log::{debug, error, info};
use tokio::sync::broadcast::Receiver;
use crate::proto::protocol::JsonMessage;
use crate::proto::transport::MiioTransport;
use paho_mqtt as mqtt;
use paho_mqtt::{AsyncClient, Message};
pub use paho_mqtt::Message as MqttMessage;
pub use paho_mqtt::types;
use serde_json::{json, Map, Value};
use tap::TapFallible;
use tokio::sync::RwLock;
use tokio::time::timeout;

pub struct OpenMiIOMqttTransport {
    pub client: AsyncClient,
    pub msg_sender: tokio::sync::broadcast::Sender<JsonMessage>,
    pub receiver: Arc<RwLock<async_channel::Receiver<Option<Message>>>>,
}

impl OpenMiIOMqttTransport {
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
        let (msg_sender, _) = tokio::sync::broadcast::channel(10);
        let strm = client.get_stream(100);
        //启动消息监听
        Ok(Self {
            client,
            msg_sender,
            receiver: Arc::new(RwLock::new(strm)),
        })
    }
    pub async fn await_result(&self, id: u32, tv: Duration) -> anyhow::Result<JsonMessage> {
        timeout(tv, async move {
            loop {
                let msg = self.recv().recv().await?;
                if let Some(val) = msg.data.get("id") {
                    if val.as_u64() == Some(id as u64) {
                        return Ok(msg);
                    }
                };
            }
        }).await?
    }
}


impl MiioTransport<Message, JsonMessage> for OpenMiIOMqttTransport {
    fn send(&self, data: Message) -> BoxFuture<anyhow::Result<()>> {
        async move {
            // let msg = mqtt::Message::new("test", "Hello Rust MQTT world!", mqtt::QOS_1);
            println!("Sending message: {:?}", data);
            self.client.publish(data).await?;
            Ok(())
        }.boxed()
    }

    /// 获取一个数据接收器
    fn recv(&self) -> Receiver<JsonMessage> {
        self.msg_sender.subscribe()
    }

    fn start_listen(&self) -> BoxFuture<()> {
        let receiver = self.receiver.clone();
        let sender = &self.msg_sender;
        async move {
            let mut a = receiver.write().await;
            while let Some(msg) = a.next().await {
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
                                    let _ = sender.send(
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
            error!("读取数据失败,客户端断开");
        }.boxed()
    }
}