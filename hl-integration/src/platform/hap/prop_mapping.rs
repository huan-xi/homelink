/// 属性映射 参数
pub struct PropMappingParam<T> {
    cid: u64,
    /// 映射参数
    param: T,
    /// 转换函数
    convert: Option<fn(serde_json::Value) -> serde_json::Value>,
}