use sea_orm::{DeriveActiveEnum, EnumIter, FromJsonQueryResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::hap::unit_convertor::eval_expr_convertor::{EvalExprConvertor, EvalExprParam};
use crate::hap::unit_convertor::kelvin_to_mired_convertor::KelvinToMiredConvertor;
use crate::hap::unit_convertor::scale_down::ScaleDownX10Conv;

pub mod eval_expr_convertor;
mod kelvin_to_mired_convertor;
mod scale_down;

pub trait Convertor {
    /// 转成目标值 hap 的值
    fn to(&self, param: Option<ConvertorParamType>, value: Value) -> anyhow::Result<Value>;
    ///
    fn from(&self, param: Option<ConvertorParamType>, value: Value) -> anyhow::Result<Value>;
    /// 是否成反比
    fn is_inverse(&self, param: Option<ConvertorParamType>) -> bool;
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct UnitConv(pub Option<UnitConvertor>, pub Option<ConvertorParamType>);

impl Default for UnitConv {
    fn default() -> Self {
        Self(None, None)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, FromJsonQueryResult)]
#[serde(tag = "type")]
pub enum ConvertorParamType {
    /// 计算表达式
    EvalExpr(EvalExprParam),
}


#[derive(EnumIter, DeriveActiveEnum, Copy, Clone, Hash, Debug, PartialEq, Eq, Serialize, Deserialize, )]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum UnitConvertor {
    /// KelvinToMired
    KelvinToMired = 1,
    /// 计算表达式
    EvalExpr = 2,
    /// 缩小10倍
    ScaleDownX10 = 3,
}

impl UnitConvertor {
    pub fn get_convertor(&self) -> Box<dyn Convertor> {
        match self {
            UnitConvertor::KelvinToMired => {
                Box::new(KelvinToMiredConvertor)
            }
            UnitConvertor::EvalExpr => {
                Box::new(EvalExprConvertor)
            }
            UnitConvertor::ScaleDownX10 => {
                Box::new(ScaleDownX10Conv)
            }
        }
    }
}