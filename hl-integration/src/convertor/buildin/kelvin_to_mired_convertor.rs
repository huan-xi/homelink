use std::sync::Arc;
use serde_json::Value;
use crate::convertor::ext::{ConvertorExt, ConvertorExtConstructor, ConvertorExtPointer};
use crate::JsonValue;

pub struct KelvinToMiredConvertor;

/// kelvin_to_mired 转换器
impl ConvertorExtConstructor for KelvinToMiredConvertor {
    fn new(param: JsonValue) -> anyhow::Result<ConvertorExtPointer> {
        Ok(Arc::new(KelvinToMiredConvertor))
    }
    fn name() -> String {
        "kelvin_to_mired".to_string()
    }
}

impl ConvertorExt for KelvinToMiredConvertor {
    fn to(&self, value: Value) -> anyhow::Result<Value> {
        Ok(value.as_number()
            .and_then(|v| v.as_i64())
            .and_then(|v| {
                let mired = 1_000_000 / v;
                Some(Value::from(mired))
            })
            .unwrap_or(Value::Null))
    }

    fn from(&self, value: Value) -> anyhow::Result<Value> {
        Ok(value.as_number()
            .and_then(|v| v.as_i64())
            .and_then(|v| {
                let kelvin = 1_000_000 / v;
                Some(Value::from(kelvin))
            })
            .unwrap_or(Value::Null))
    }
}