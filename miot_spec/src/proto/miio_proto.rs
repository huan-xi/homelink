use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use futures_util::future::join;
use hex::FromHex;
use impl_new::New;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::broadcast;
use crate::proto::protocol::JsonMessage;

// pub type MiotSpecProtocolPointer = Arc<Box<dyn MiotSpecProtocol + Send + Sync + 'static>>;
pub type MiotSpecProtocolPointer = Arc<dyn MiotSpecProtocol + Send + Sync + 'static>;
pub type MsgCallback = Box<dyn Fn(JsonMessage) + Send + Sync>;

/// 米家协议 发送和接收miio 指令
#[async_trait::async_trait]
pub trait MiotSpecProtocol {
    /// 获取一个新的指令id
    fn incr_cmd_id(&self) -> u64;
    /// 发送数据
    async fn send<'a>(&'a self, cmd: &'a str) -> anyhow::Result<()>;
    /// 获取一个数据接收器
    fn recv(&self) -> broadcast::Receiver<JsonMessage>;
    /// 等待结果
    async fn await_result<'a>(&'a self, id: u64, timeout_val: Option<Duration>) -> anyhow::Result<JsonMessage>;
    /// 开始监听
    async fn start_listen(&self);


    async fn get_property_timeout(&self, param: MiotSpecDTO, timeout_val: Option<Duration>) -> anyhow::Result<Option<Value>> {
        let mut values = self.get_properties(vec![param], timeout_val).await?;
        if values.len() == 0 {
            return Ok(None);
        };
        Ok(values.remove(0).value)
    }
    /// 读取属性值
    async fn get_property(&self, param: MiotSpecDTO) -> anyhow::Result<Option<Value>> {
        return self.get_property_timeout(param, None).await;
    }
    async fn set_property(&self, param: MiotSpecDTO) -> anyhow::Result<()> {
        self.set_property_timeout(param, None).await
    }
    async fn set_property_timeout(&self, param: MiotSpecDTO, timeout_val: Option<Duration>) -> anyhow::Result<()> {
        let mut values = self.set_properties(vec![param], timeout_val).await?;
        values.remove(0).value;
        Ok(())
    }

    /// 调用rpc
    async fn call_rpc(&self, method: &str, params: Vec<MiotSpecDTO>, timeout: Option<Duration>) -> anyhow::Result<JsonMessage> {
        let id = self.incr_cmd_id();
        let param = serde_json::json![{
            "id":id,
            "method":method,
            "params":params
        }];
        let str = param.to_string();
        info!("call_rpc:{}", str);
        let sender = self.send(str.as_str());
        let recv = self.await_result(id, timeout);
        let (r1, r2) = join(recv, sender).await;
        r2?;
        r1
    }

    async fn set_properties(&self, params: Vec<MiotSpecDTO>, timeout_val: Option<Duration>) -> anyhow::Result<Vec<MiotSpecDTO>> {
        info!("set_properties value:{:?}", params);
        let mut result = self.call_rpc("set_properties", params, timeout_val).await?;
        let value = result.data.remove("result")
            .ok_or(anyhow::anyhow!("无result 节点"))?;
        let miot_specs: Vec<MiotSpecDTO> = serde_json::from_value(value)?;
        //value 转成 MiotSpecDTO

        Ok(miot_specs)
    }
    async fn get_properties(&self, params: Vec<MiotSpecDTO>, timeout_val: Option<Duration>) -> anyhow::Result<Vec<MiotSpecDTO>> {
        let mut result = self.call_rpc("get_properties", params, timeout_val).await?;
        let value = result.data.remove("result")
            .ok_or(anyhow::anyhow!("properties 无result"))?;
        let miot_specs: Vec<MiotSpecDTO> = serde_json::from_value(value)?;
        //value 转成 MiotSpecDTO
        info!("get_properties result:{:?}", miot_specs);
        Ok(miot_specs)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, New)]
pub struct MiotSpecId {
    pub siid: i32,
    pub piid: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, New)]
pub struct MiotSpecDTO {
    pub did: String,
    pub siid: i32,
    pub piid: i32,
    pub value: Option<Value>,
}
