use std::time::Duration;
use tokio::sync::broadcast::Receiver;
use crate::proto::miio_proto::{MiotSpecProtocol, MiotSpecProtocolPointer};
use crate::proto::protocol::JsonMessage;

/// 蓝牙数据属性映射协议
/// 蓝牙设备需要通过网关设备去调用
pub struct BleMappingProto {
    gateway: MiotSpecProtocolPointer,
}

impl BleMappingProto {
    pub fn new(gateway: MiotSpecProtocolPointer) -> Self {
        Self {
            gateway
        }
    }
}
#[async_trait::async_trait]
impl MiotSpecProtocol for BleMappingProto {
    fn incr_cmd_id(&self) -> u64 { self.gateway.incr_cmd_id() }

    async fn send<'a>(&'a self, cmd: &'a str) -> anyhow::Result<()> {
        self.gateway.send(cmd).await
    }

    fn recv(&self) -> Receiver<JsonMessage> {
        todo!("暂时无法接受蓝牙数据")
    }

    async fn await_result<'a>(&'a self, id: u64, timeout_val: Option<Duration>) -> anyhow::Result<JsonMessage> {
        self.gateway.await_result(id, timeout_val).await
    }

    async fn start_listen(&self) -> () {
        todo!("蓝牙设备暂时不需要监听")
    }
}