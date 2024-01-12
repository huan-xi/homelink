use serde_json::Value;
use crate::hap::unit_convertor::{Convertor, ConvertorParamType};

pub struct KelvinToMiredConvertor;

impl Convertor for KelvinToMiredConvertor {
    fn to(&self, param: Option<ConvertorParamType>, value: Value) -> anyhow::Result<Value> {
        Ok(value.as_number()
            .and_then(|v| v.as_i64())
            .and_then(|v| {
                let mired = 1_000_000 / v;
                Some(Value::from(mired))
            })
            .unwrap_or(Value::Null))
    }

    fn from(&self, param: Option<ConvertorParamType>, value: Value) -> anyhow::Result<Value> {
        Ok(value.as_number()
            .and_then(|v| v.as_i64())
            .and_then(|v| {
                let kelvin = 1_000_000 / v;
                Some(Value::from(kelvin))
            })
            .unwrap_or(Value::Null))
    }
}