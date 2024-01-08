use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use futures_util::{SinkExt, StreamExt};
use crate::proto::transport::MiioTransport;
use hex::FromHex;
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::time::timeout;
use crate::proto::protocol::{JsonMessage, Message, MessageHeader};
use crate::proto::transport::gateway_mqtt::MqttMessage;
use crate::proto::transport::Transport;


/// 米家协议 发送和接收miio 指令

pub struct MiIOProtocol {
    /// Socket
    transport: Transport,
    id: AtomicU32,
    /// Device token
    timeout: Duration,
}

unsafe impl Send for MiIOProtocol {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiotSpecDTO {
    pub did: String,
    pub siid: i32,
    pub piid: i32,
    pub value: Option<Value>,
}

impl MiIOProtocol {
    pub async fn new(transport: Transport) -> anyhow::Result<Self> {
        //开启数据监听协程
        Ok(Self {
            id: AtomicU32::new(1),
            timeout: Duration::from_secs(2),
            transport,
        })
    }

    /// 监听事件
    pub async fn start_listen(&self) {
        match &self.transport {
            Transport::Udp(udp) => { udp.start_listen().await }
            Transport::OpenMiIOMqtt(mqtt) => { mqtt.start_listen().await }
        }
    }

    /*    /// 握手
        pub async fn handshake(&self) -> anyhow::Result<()> {
            let msg = MiIOProtocol::discover(Some(self.ip.as_str()), Duration::from_secs(5)).await?;
            // self.stamp.write().await.replace(msg.header.stamp);
            Ok(())
        }*/

    /// 发送命令
    pub async fn send(&self, cmd: &str) -> anyhow::Result<()> {
        match &self.transport {
            Transport::Udp(trans) => {
                let id = self.id.load(Ordering::SeqCst);
                let msg = trans.build_message(id, cmd).await?;
                let bytes = msg.pack_to_vec();
                // info!("send msg:{:?}",msg);
                trans.send(bytes).await?;
            }
            Transport::OpenMiIOMqtt(trans) => {
                let msg = MqttMessage::new("miio/command", cmd, paho_mqtt::QOS_1);
                //发送 "miio/command",
                trans.send(msg).await?;
            }
        }
        Ok(())
    }

    /// 读取属性值
    pub async fn get_property(&self, param: MiotSpecDTO) -> anyhow::Result<Option<Value>> {
        let mut values = self.get_properties(vec![param], None).await?;
        if values.len() == 0 {
            return Ok(None);
        };
        Ok(values.remove(0).value)
    }
    pub async fn set_property(&self, param: MiotSpecDTO) -> anyhow::Result<()> {
        let mut values = self.set_properties(vec![param], None).await?;
        // values.remove(0).value;
        Ok(())
    }

    /// 等待结果
    pub async fn await_result(&self, id: u32, timeout_val: Option<Duration>) -> anyhow::Result<JsonMessage> {
        let t = match timeout_val {
            None => {
                self.timeout
            }
            Some(s) => { s }
        };
        match &self.transport{
            Transport::Udp(udp) => {
                udp.await_result(id, t).await
            }
            Transport::OpenMiIOMqtt(mqtt) => {
                mqtt.await_result(id, t).await
            }
        }
    }

    pub async fn call_rpc(&self, method: &str, params: Vec<MiotSpecDTO>, timeout: Option<Duration>) -> anyhow::Result<JsonMessage> {
        let id = self.id.fetch_add(1, Ordering::SeqCst);
        let param = serde_json::json![{
            "id":id,
            "method":method,
            "params":params
        }];
        let str = param.to_string();
        info!("call_rpc:{}", str);
        self.send(str.as_str()).await?;
        self.await_result(id, timeout).await
    }

    pub async fn set_properties(&self, params: Vec<MiotSpecDTO>, timeout: Option<Duration>) -> anyhow::Result<Vec<MiotSpecDTO>> {
        info!("set_properties value:{:?}", params);
        let mut result = self.call_rpc("set_properties", params, timeout).await?;
        let value = result.data.remove("result")
            .ok_or(anyhow::anyhow!("无result 节点"))?;
        let miot_specs: Vec<MiotSpecDTO> = serde_json::from_value(value)?;
        //value 转成 MiotSpecDTO

        Ok(miot_specs)
    }
    pub async fn get_properties(&self, params: Vec<MiotSpecDTO>, timeout: Option<Duration>) -> anyhow::Result<Vec<MiotSpecDTO>> {
        let mut result = self.call_rpc("get_properties", params, timeout).await?;
        let value = result.data.remove("result")
            .ok_or(anyhow::anyhow!("properties 无result"))?;
        let miot_specs: Vec<MiotSpecDTO> = serde_json::from_value(value)?;
        //value 转成 MiotSpecDTO
        info!("get_properties result:{:?}", miot_specs);
        Ok(miot_specs)
    }

    pub fn get_transport(&self) -> &Transport {
        &self.transport
    }
}