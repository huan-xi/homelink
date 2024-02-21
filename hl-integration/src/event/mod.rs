pub mod events;
pub mod emitter;

use futures_util::future::BoxFuture;
use crate::event::events::{DeviceEvent, DeviceEventPointer};


/// 事件监听器
/// 执行一个异步任务
pub type EventListener = Box<dyn (Fn(DeviceEventPointer) -> BoxFuture<'static, ()>) + Send + Sync>;


/// 事件监听器设备
#[async_trait::async_trait]
pub trait HlDeviceListenable {
    /// 设备添加事件监听器
    async fn add_listener(&self, listener: EventListener) -> i64;
     fn remove_listener(&self, id: i64) -> i64;
}
