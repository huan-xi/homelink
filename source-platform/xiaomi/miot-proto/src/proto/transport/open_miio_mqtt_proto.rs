use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use anyhow::anyhow;

use log::{debug, error};
// use paho_mqtt::{AsyncClient, Message};
// use paho_mqtt as mqtt;
// pub use paho_mqtt::Message as MqttMessage;
use rumqttc::{AsyncClient, Event, EventLoop, Incoming, MqttOptions, QoS};
use serde_json::{Map, Value};
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;
use tokio::time::timeout;

use crate::proto::miio_proto::MiotSpecProtocol;
use crate::proto::protocol::JsonMessage;

/// 基于 miio 的网关传输协议
pub struct OpenMiIOMqttSpecProtocol {
    pub client: AsyncClient,
    event_loop: Mutex<EventLoop>,
    // pub receiver: Arc<RwLock<async_channel::Receiver<Option<Message>>>>,
    msg_sender: Sender<JsonMessage>,
    id: AtomicU64,
    /// Device token
    timeout: Duration,
}

impl OpenMiIOMqttSpecProtocol {
    pub async fn new(ip: &str, port: u32) -> anyhow::Result<Self> {
        let mut mqttoptions = MqttOptions::new("homelink", ip, port as u16);
        mqttoptions
            .set_keep_alive(Duration::from_secs(5))
            .set_manual_acks(false);

        let (client, event_loop) = AsyncClient::new(mqttoptions, 10);
        client.subscribe("#", QoS::AtMostOnce).await?;


        let (msg_sender, _) = tokio::sync::broadcast::channel(2048);
        // let strm = client.get_stream(100);
        //启动消息监听
        Ok(Self {
            client,
            event_loop: Mutex::new(event_loop),
            msg_sender,
            id: Default::default(),
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
        self.client.publish("miio/command", QoS::AtLeastOnce, false, cmd).await?;
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
                    if val.as_u64() == Some(id) {
                        debug!("await_result:{:?}", msg.data);
                        return Ok(msg);
                    }
                };
            }
        }).await.map_err(|_e| anyhow!("执行命令超时"))?
    }

    async fn start_listen(&self) {
        match self.event_loop.try_lock() {
            Ok(mut event_loop) => {
                while let Ok(notification) = event_loop.poll().await {
                    match notification {
                        Event::Incoming(incoming) => {
                            if let Incoming::Publish(msg) = incoming {
                                let topic = msg.topic.as_str();
                                // debug!("Received topic:{} message: {}",topic, msg.payload.len());
                                match String::from_utf8(msg.payload.to_vec()) {
                                    Ok(str) => {
                                        match topic {
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
                                    }
                                    Err(e) => {
                                        error!("解析数据失败: {:?}", e);
                                    }
                                }
                            };
                        }
                        Event::Outgoing(_) => {}
                    }

                }
            }
            Err(_e) => {
                error!("获取event_loop 失败");
            }
        }


        // let receiver = self.receiver.clone();
        // let sender = &self.msg_sender;
        // let mut write = receiver.write().await;
    /*    while let Some(msg) = write.next().await {
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
        }*/
        error!("读取数据失败,mqtt 客户端断开");
    }
}


#[cfg(test)]
mod test {
    use log::info;

    #[tokio::test]
    async fn test_mqttc() {
        std::env::set_var("RUST_LOG", "debug");
        env_logger::init();

        use rumqttc::{AsyncClient, MqttOptions, QoS};
        use tokio::{task, time};
        use std::time::Duration;

        let mut mqttoptions = MqttOptions::new("rumqtt-async", "192.168.68.24", 1883);
        mqttoptions.set_keep_alive(Duration::from_secs(5));

        let (mut client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
        client.subscribe("#", QoS::AtMostOnce).await.unwrap();

        task::spawn(async move {
            for i in 0..10 {
                client.publish("hello/rumqtt", QoS::AtLeastOnce, false, vec![i; i as usize]).await.unwrap();
                time::sleep(Duration::from_millis(100)).await;
            }
        });

        while let Ok(notification) = eventloop.poll().await {
            info!("Received = {:?}", notification);
        }
    }
}