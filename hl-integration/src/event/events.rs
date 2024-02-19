/// 设备产生的事件
#[derive(Debug,Clone)]
pub enum DeviceEvent {
    /// 属性变化事件
    PropertyChanged(serde_json::Value),
}