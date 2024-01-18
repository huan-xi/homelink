use std::fmt::Debug;
use std::sync::Arc;
use futures_util::future::{BoxFuture, join_all};
use impl_new::New;
use log::{debug, info};
use serde::Serialize;
use crate::device::ble::value_types::BleValue;
use crate::proto::miio_proto::MiotSpecDTO;
use crate::proto::protocol::JsonMessage;

/*#[derive(Debug, Clone, New)]
pub struct DeviceEvent {
    device_id: String,
    event_type: EventType,
}*/

///监听数据的类型
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum EventType {
    /// 属性更新事件
    UpdateProperty(MiotSpecDTO),
    /// 属性设置事件
    SetProperty(MiotSpecDTO),
    /// 网关消息
    GatewayMsg(JsonMessage),
}

pub type DataListener<T> = Box<dyn (Fn(T) -> BoxFuture<'static, anyhow::Result<()>>) + Send + Sync>;


#[derive(Default)]
pub struct DataEmitter<T> {
    has_js_listener: bool,
    listeners: Vec<DataListener<T>>,
}

impl<T> DataEmitter<T>
    where T: Clone + Send + Sync + Debug {
    pub fn new() -> DataEmitter<T> { DataEmitter { has_js_listener: false, listeners: vec![] } }

    pub fn add_listener(&mut self, listener: DataListener<T>) {
        self.listeners.push(listener);
    }
    pub fn is_empty(&self) -> bool {
        self.listeners.is_empty()
    }

    pub async fn emit(&self, event: T) {
        join_all(self.listeners.iter().map(|listener| listener(event.clone()))).await;
    }
}