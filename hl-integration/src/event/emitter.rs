use std::sync::Arc;
use tokio::sync::RwLock;
use crate::event::EventListener;
use crate::event::events::DeviceEvent;

/// 事件提交器
#[derive(Default)]
pub struct EventEmitter {
    listeners: RwLock<Vec<Arc<EventListener>>>,
}

impl EventEmitter {

    /// 添加一个事件监听器
    pub async fn add_listener(&self, listener: EventListener) {
        self.listeners.write().await.push(Arc::new(listener));
    }

    /// 提交一个事件
    pub async fn emit(&self, event: DeviceEvent) {
        self.listeners
            .read()
            .await
            .iter()
            .for_each(|i| {
                let listener = i.clone();
                let event = event.clone();
                tokio::spawn(listener(event));
            });
    }
}