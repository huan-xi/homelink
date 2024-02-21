use std::sync::Arc;
use std::sync::atomic::AtomicI64;
use dashmap::DashMap;
use tokio::sync::RwLock;
use crate::event::EventListener;
use crate::event::events::{DeviceEvent, DeviceEventPointer};

/// 事件提交器
#[derive(Default)]
pub struct DeviceEventEmitter {
    id: AtomicI64,
    listeners: DashMap<i64, Arc<EventListener>>,
}

impl DeviceEventEmitter {
    pub fn is_empty(&self) -> bool {
        self.listeners.is_empty()
    }
    /// 添加一个事件监听器
    pub async fn add_listener(&self, listener: EventListener) -> i64 {
        let id = self.id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.listeners.insert(id, Arc::new(listener));
        id
    }
    /// 移除一个事件监听器
    pub fn remove_listener(&self, id: i64) -> i64 {
        self.listeners.remove(&id);
        id
    }

    /// 提交一个事件
    pub async fn emit(&self, event: DeviceEventPointer) {
        self.listeners
            .iter()
            .for_each(|i| {
                let listener = i.clone();
                let event = event.clone();
                tokio::spawn(listener(event));
            });
    }
}