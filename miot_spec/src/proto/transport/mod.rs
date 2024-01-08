use futures_util::future::BoxFuture;
use futures_util::{FutureExt, SinkExt};
use crate::proto::protocol::{JsonMessage, RecvMessage};
use crate::proto::transport::gateway_mqtt::OpenMiIOMqttTransport;
use crate::proto::transport::miio_udp_transport::UdpTransport;

pub mod miio_udp_transport;
pub mod gateway_mqtt;


/// 传输层协议
pub enum Transport {
    /// udp 协议
    Udp(UdpTransport),
    /// 通过网关的mqtt 协议
    OpenMiIOMqtt(OpenMiIOMqttTransport),
}


/// 传输层 Vec<u8>
pub trait MiioTransport<T, F: RecvMessage> {
    /// 发送数据
    fn send(&self, data: T) -> BoxFuture<anyhow::Result<()>>;
    /// 获取一个数据接收器
    fn recv(&self) -> tokio::sync::broadcast::Receiver<F>;

    /// 开始监听消息
    fn start_listen(&self) -> BoxFuture<()>;
    // 设置数据监听器
    // 必须要运行设备
}

