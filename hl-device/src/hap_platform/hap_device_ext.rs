use hap::characteristic::{CharReadParam, CharUpdateParam, ReadCharResults, UpdateCharResults};
use crate::event::HlDeviceListenable;


/// 实现该trait可以转成homekit 配件
/// 特征值读写,提交事件 能力
/// 初始化绑定,
#[async_trait::async_trait]
pub trait HapDeviceExt: HlDeviceListenable {
    /// hap设备特征值批量写入
    /// 配件读写值
    async fn read_chars_value(&self, params: Vec<CharReadParam>) -> ReadCharResults;
    async fn update_chars_value(&self, params: Vec<CharUpdateParam>) -> UpdateCharResults;
}