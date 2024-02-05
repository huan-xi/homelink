use std::sync::Arc;
use std::time::Duration;
use anyhow::anyhow;
use impl_new::New;
use tokio::sync::broadcast::Receiver;
use tokio::sync::RwLock;
use crate::cloud::MiCloud;
use crate::proto::miio_proto::{METHOD_GET_PROPERTIES, METHOD_SET_PROPERTIES, MiotSpecDTO, MiotSpecProtocol};
use crate::proto::protocol::JsonMessage;


#[derive(New)]
pub struct CloudMiioProto {
    pub cloud_client: Arc<RwLock<MiCloud>>,
}

#[async_trait::async_trait]
impl MiotSpecProtocol for CloudMiioProto {
    fn incr_cmd_id(&self) -> u64 {
        todo!()
    }

    async fn send<'a>(&'a self, _cmd: &'a str) -> anyhow::Result<()> {
        Ok(())
    }

    fn recv(&self) -> Receiver<JsonMessage> {
        todo!("can not recv")
    }

    async fn await_result<'a>(&'a self, _id: u64, _timeout_val: Option<Duration>) -> anyhow::Result<JsonMessage> {
        todo!("can not await_result")
    }

    async fn start_listen(&self) {
        todo!("can not start_listen")
    }

    async fn call_rpc(&self, method: &str, params: Vec<MiotSpecDTO>, _timeout: Option<Duration>) -> anyhow::Result<JsonMessage> {
        let url = match method {
            METHOD_GET_PROPERTIES => "/miotspec/prop/get",
            METHOD_SET_PROPERTIES => "/miotspec/prop/set",
            // /miotspec/action
            _ => {
                return Err(anyhow!("不支持的方法:{}", method));
            }
        };

        let str = serde_json::json!({
            "params": params,
        }).to_string();
        // "/miotspec/prop/get",
        let result = self.cloud_client.read().await.call_api(url, str.as_str()).await?;
        let mut map = result.as_object().ok_or(anyhow!("返回结果不是json对象"))?.clone();
        let code = map.remove("code")
            .and_then(|v| v.as_u64())
            .ok_or(anyhow!("返回结果没有code"))?;
        if code != 0 {
            return Err(anyhow!("返回结果错误"));
        }
        return Ok(JsonMessage::new(map));
    }
}