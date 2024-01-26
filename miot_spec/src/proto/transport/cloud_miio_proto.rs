use std::sync::Arc;
use std::time::Duration;
use anyhow::anyhow;
use impl_new::New;
use tokio::sync::broadcast::Receiver;
use crate::cloud::MiCloud;
use crate::proto::miio_proto::{MiotSpecDTO, MiotSpecProtocol};
use crate::proto::protocol::JsonMessage;

#[derive(New)]
pub struct CloudMiioProto {
    pub cloud_client: Arc<MiCloud>,
}

#[async_trait::async_trait]
impl MiotSpecProtocol for CloudMiioProto {
    fn incr_cmd_id(&self) -> u64 {
        todo!()
    }

    async fn send<'a>(&'a self, cmd: &'a str) -> anyhow::Result<()> {
        Ok(())
    }

    fn recv(&self) -> Receiver<JsonMessage> {
        todo!()
    }

    async fn await_result<'a>(&'a self, id: u64, timeout_val: Option<Duration>) -> anyhow::Result<JsonMessage> {
        todo!()
    }

    async fn start_listen(&self) {
        todo!()
    }

    async fn call_rpc(&self, method: &str, params: Vec<MiotSpecDTO>, timeout: Option<Duration>) -> anyhow::Result<JsonMessage> {
        /*match method {
            _ => todo!()
        }*/
        let str = serde_json::json!({
            "params": params,
        }).to_string();
        let result = self.cloud_client.call_api("/miotspec/prop/get", str.as_str()).await?;
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