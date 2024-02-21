use std::sync::Arc;
use crate::JsonValue;
pub type ConvertorExtPointer = Arc<dyn ConvertorExt + Send + Sync + 'static>;

pub trait ConvertorExtConstructor {
    fn new(param: JsonValue) -> anyhow::Result<ConvertorExtPointer>;
    fn name() -> String;
}

/// 转换器扩展
pub trait ConvertorExt{
    /// 转成目标值 target 的值
    fn to(&self, value: JsonValue) -> anyhow::Result<JsonValue>;

    /// 从来源平台读值
    fn from(&self, value: JsonValue) -> anyhow::Result<JsonValue>;
}