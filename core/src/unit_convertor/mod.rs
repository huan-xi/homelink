use sea_orm::{DeriveActiveEnum, EnumIter, FromJsonQueryResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::unit_convertor::eval_expr_convertor::{EvalExprConvertor, EvalExprParam};
use crate::unit_convertor::kelvin_to_mired_convertor::KelvinToMiredConvertor;
use crate::unit_convertor::scale_down::ScaleDownX10Conv;

pub mod eval_expr_convertor;
mod kelvin_to_mired_convertor;
mod scale_down;
// mod factory;

/// 单位转换器
pub struct UnitConvertor;


pub trait ConvertorExt {
    /// 转成目标值 hap_platform 的值
    fn to(&self, param: Option<ConvertorParamType>, value: Value) -> anyhow::Result<Value>;
    /// 从来源平台读值
    fn from(&self, param: Option<ConvertorParamType>, value: Value) -> anyhow::Result<Value>;
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct UnitConv(pub Option<UnitConvertorType>, pub Option<ConvertorParamType>);

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
pub enum UnitConvertorType {
    /// KelvinToMired
    KelvinToMired = 1,
    /// 计算表达式
    EvalExpr = 2,
    /// 缩小10倍
    ScaleDownX10 = 3,
}

// pub struct UnitConvertor;


impl UnitConvertorType {
    pub fn get_convertor(&self) -> Box<dyn ConvertorExt> {
        match self {
            UnitConvertorType::KelvinToMired => {
                Box::new(KelvinToMiredConvertor)
            }
            UnitConvertorType::EvalExpr => {
                Box::new(EvalExprConvertor)
            }
            UnitConvertorType::ScaleDownX10 => {
                Box::new(ScaleDownX10Conv)
            }
        }
    }
}