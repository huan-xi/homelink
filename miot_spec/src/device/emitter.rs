use std::fmt::Debug;
use std::sync::Arc;
use futures_util::future::{BoxFuture, join_all};
use log::{debug, info};
use crate::device::ble::value_types::BleValue;
use crate::proto::miio_proto::MiotSpecDTO;
use crate::proto::protocol::JsonMessage;

///监听数据的类型
#[derive(Debug, Clone)]
pub enum EventType {
    /// 属性更新事件
    UpdateProperty(MiotSpecDTO),
    /// 属性设置事件
    SetProperty(MiotSpecDTO),
    /// 网关消息
    GatewayMsg(JsonMessage),
}

// pub trait DataListener<T: Default + Clone + Serialize + Send + Sync>: FnMut(T) -> BoxFuture<'static, anyhow::Result<()>> + 'static + Send + Sync {}
// pub type DataListener<T> =Box< dyn FnOnce(T) -> BoxFuture<'static, anyhow::Result<()>> + Send + Sync>;
pub type DataListener<T> = Box<dyn (Fn(T) -> BoxFuture<'static, anyhow::Result<()>>) + Send + Sync>;
// pub type DataListener<T> = Box<dyn FnOnce(T) -> BoxFuture<'static, anyhow::Result<()>> + Send + Sync>;


#[derive(Default)]
pub struct DataEmitter<T> {
    listeners: Vec<DataListener<T>>,
}

impl<T> DataEmitter<T>
    where T: Clone + Send + Sync + Debug {
    pub fn new() -> DataEmitter<T> { DataEmitter { listeners: vec![] } }

    pub async fn add_listener(&mut self, listener: DataListener<T>) {
        self.listeners.push(listener);
    }
    pub fn is_empty(&self) -> bool {
        self.listeners.is_empty()
    }

    pub async fn emit(&self, event: T) {
        join_all(self.listeners.iter().map(|listener| listener(event.clone()))).await;
    }
}