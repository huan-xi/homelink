use serde_json::Value;
use crate::hap::unit_convertor::{Convertor, ConvertorParamType};

pub struct ScaleDownX10Conv;
impl Convertor for ScaleDownX10Conv {
    fn to(&self, param: Option<ConvertorParamType>, value: Value) -> anyhow::Result<Value> {
        Ok(value.as_f64()
            .map(|v| v / 10.0)
            .map(|v| Value::from(v))
            .unwrap_or(Value::Null))
    }

    fn from(&self, param: Option<ConvertorParamType>, value: Value) -> anyhow::Result<Value> {
        Ok(value.as_f64()
            .map(|v| v * 10.0)
            .map(|v| Value::from(v)).unwrap_or(Value::Null))
    }

    fn is_inverse(&self, param: Option<ConvertorParamType>) -> bool {
        return false;
    }
}

#[test]
pub fn test() {
    let convertor = ScaleDownX10Conv {};
    let value = convertor.to(None, Value::from(150)).unwrap();
    println!("value: {:?}", value);
    assert_eq!(value.as_f64().unwrap(), 15.0);

}