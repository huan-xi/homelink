use std::collections::HashMap;
use anyhow::anyhow;
use once_cell::sync::OnceCell;
use crate::convertor::buildin::kelvin_to_mired_convertor::KelvinToMiredConvertor;
use crate::convertor::ext::{ConvertorExtConstructor, ConvertorExtPointer};
use crate::JsonValue;

pub static CONVERTOR_FACTORY: OnceCell<UnitConvertorFactory> = OnceCell::new();

pub type ConvertorConstructorFunc = fn(param: JsonValue) -> anyhow::Result<ConvertorExtPointer>;
/// 单位转换器工厂

pub struct UnitConvertorFactory {
    convertors: HashMap<String, ConvertorConstructorFunc>,
}

impl UnitConvertorFactory {
    pub fn get_convertor(&self, name: &str, param: Option<JsonValue>) -> anyhow::Result<ConvertorExtPointer> {
        let constructor = self.convertors.get(name)
            .ok_or_else(|| anyhow!("convertor {} not found", name))?;
        constructor(param.unwrap_or(JsonValue::Null))
    }
}

impl Default for UnitConvertorFactory {
    fn default() -> Self {
        let mut convertors = HashMap::new();
        //注册内置
        convertors.insert(KelvinToMiredConvertor::name(), KelvinToMiredConvertor::new as ConvertorConstructorFunc);
        Self {
            convertors,
        }
    }
}

pub fn get_unit_convertor_factory() -> &'static UnitConvertorFactory {
    CONVERTOR_FACTORY.get_or_init(|| UnitConvertorFactory::default())
}
