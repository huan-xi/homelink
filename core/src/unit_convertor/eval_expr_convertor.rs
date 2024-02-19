use evalexpr::{ContextWithMutableVariables, eval_with_context_mut, HashMapContext, Value as EvalValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::unit_convertor::{ConvertorExt, ConvertorParamType};

/// 表达式转换
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct EvalExprParam {
    /// 转成hap定值
    pub to_expr: String,
    /// 从hap 转成其他类型
    pub from_expr: String,
    /// 是否是成反比
    pub is_inverse: bool,
}

/// val+1
pub struct EvalExprConvertor;


pub fn eval_expr(param: Option<ConvertorParamType>, value: Value, expr_func: fn(EvalExprParam) -> String) -> anyhow::Result<Value> {
    if let Some(ConvertorParamType::EvalExpr(param)) = param {
        let mut context = HashMapContext::new();
        context.set_value("val".to_string(), json_value_to_eval_value(value)?)?;
        let expr = expr_func(param);
        let val = eval_with_context_mut(expr.as_str(), &mut context)?;
        return Ok(json_value_from_eval_value(val)?);
    }
    return Err(anyhow::anyhow!("不支持该类型"));
}

impl ConvertorExt for EvalExprConvertor {
    fn to(&self, param: Option<ConvertorParamType>, value: Value) -> anyhow::Result<Value> {
        eval_expr(param, value, |param| param.to_expr)
    }

    fn from(&self, param: Option<ConvertorParamType>, value: Value) -> anyhow::Result<Value> {
        eval_expr(param, value, |param| param.from_expr)
    }
}


fn json_value_to_eval_value(value: Value) -> anyhow::Result<EvalValue> {
    match value {
        Value::Null => Ok(EvalValue::Empty),
        Value::Bool(v) => Ok(EvalValue::Boolean(v)),
        Value::Number(v) => {
            if v.is_i64() {
                Ok(EvalValue::Int(v.as_i64().unwrap()))
            } else if v.is_f64() {
                Ok(EvalValue::Float(v.as_f64().unwrap()))
            } else {
                Err(anyhow::anyhow!("not support number type"))
            }
        }
        Value::String(v) => Ok(EvalValue::String(v)),
        Value::Array(v) => {
            let mut vec = Vec::new();
            for item in v {
                vec.push(json_value_to_eval_value(item)?);
            }
            Ok(EvalValue::Tuple(vec))
        }
        Value::Object(v) => {
            todo!();
            /*    let mut map = std::collections::HashMap::new();
                for (k, v) in v {
                    map.insert(k, json_value_to_eval_value(v)?);
                }
                Ok(EvalValue::Tuple(map))*/
        }
    }
}


fn json_value_from_eval_value(value: EvalValue) -> anyhow::Result<Value> {
    match value {
        EvalValue::Empty => Ok(Value::Null),
        EvalValue::Boolean(v) => Ok(Value::Bool(v)),
        EvalValue::Int(v) => Ok(Value::Number(serde_json::Number::from(v))),
        EvalValue::Float(v) => Ok(Value::Number(serde_json::Number::from_f64(v).unwrap())),
        EvalValue::String(v) => Ok(Value::String(v)),
        EvalValue::Tuple(v) => {
            let mut vec = Vec::new();
            for item in v {
                vec.push(json_value_from_eval_value(item)?);
            }
            Ok(Value::Array(vec))
        }
    }
}

