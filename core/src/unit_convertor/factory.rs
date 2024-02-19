use once_cell::sync::OnceCell;

pub static CONVERTOR_FACTORY: OnceCell<ConvertorFactory> = OnceCell::new();

///转换器工厂
pub struct ConvertorFactory {
    // pub fn get(&self, name: &str) -> Option<ConvertorConstructorFunc> {
    //     self.convertor_map.get(name).map(|v| v.value().clone())
    // }
}