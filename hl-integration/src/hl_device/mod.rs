pub mod manager;

use std::cmp::{max, min};
use tokio::sync::Mutex;
use crate::error::DeviceExitError;
use crate::event::HlDeviceListenable;

/// 平台的设备抽象
/// 属性(特征),服务,事件
/// 读写特征值,调用服务,订阅事件
#[async_trait::async_trait]
pub trait HlDevice: Send + Sync + HlDeviceListenable {
    fn dev_id(&self) -> String;
    fn device_type(&self) -> &str;

    /// 运行设备
    /// 退出时判断是否可重试
    async fn run(&self) -> Result<(), Box<dyn DeviceExitError>>;

    fn retry_info(&self) -> &RetryInfo;
}


/// 重试信息

pub struct RetryInfo {
    /// 重试次数
    pub retry_count: Mutex<u32>,
    /// 最大重试间隔 5 分钟,单位毫秒
    pub max_interval: u32,
}

impl Default for RetryInfo {
    fn default() -> Self {
        Self {
            retry_count: Mutex::new(0),
            max_interval: 60_0000,
        }
    }
}

impl RetryInfo {
    pub async fn incr(&self) -> u32 {
        let mut write = self.retry_count.lock().await;
        *write += 1;
        *write
    }
    pub async fn reset(&self) {
        let mut write = self.retry_count.lock().await;
        *write = 0;
    }
    pub async fn get(&self) -> u32 {
        let count_read = self.retry_count.lock().await;
        // 产生1-1000 随机数
        let rand = rand::random::<u32>() % 1000 + 1;
        let t = 2u32.checked_pow(*count_read - 1)
            .and_then(|x| x.checked_mul(1000))
            .unwrap_or(self.max_interval);
        min(t, self.max_interval) + rand
    }
}