use once_cell::sync::OnceCell;
use std::collections::HashMap;

use sea_orm::{ActiveEnum, ActiveModelTrait, ColumnTrait, ColumnType, JsonValue};
use crate::db::entity::hap_accessory::Relation::HapBridge;
use crate::db::entity::hap_bridge::BridgeCategory;

use crate::db::entity::iot_device::DeviceType;
use crate::db::entity::prelude::{HapAccessoryColumn, HapAccessoryEntity, HapBridgeColumn, IotDeviceActiveModel, IotDeviceColumn};

pub type DeserializedEnumFunc = fn(&str) -> anyhow::Result<sea_orm::Value>;

/// 存储字段的枚举值
#[derive(Hash, Eq, PartialEq)]
pub struct ColumnEnumKey {
    entity_name: String,
    column_name: String,
}


impl ColumnEnumKey {
    pub fn new<T: ColumnTrait>(column: T) -> Self {
        Self {
            entity_name: column.entity_name().to_string(),
            column_name: column.as_str().to_string(),
        }
    }
}

pub struct EnumFactory {
    map: HashMap<ColumnEnumKey, DeserializedEnumFunc>,
}

impl EnumFactory {
    pub fn get_enum<T: ColumnTrait>(&self, column: T) -> Option<&DeserializedEnumFunc> {
        let key = ColumnEnumKey::new(column);
        self.map.get(&key).map(|f| f)
    }
}
macro_rules! insert_deserialized_enum_func {
    ($map:ident, $column:expr, $enum_type:ty) => {
        $map.insert(ColumnEnumKey::new($column), (|s: &str| {
            let value: $enum_type = serde_json::from_str(&format!("\"{}\"", s))?;
            Ok(value.into())
        }) as DeserializedEnumFunc);
    };
}

pub static ENUM_FACTORY: OnceCell<EnumFactory> = OnceCell::new();

pub fn get_enum_factory() -> &'static EnumFactory {
    ENUM_FACTORY.get_or_init(|| {
        let mut map = HashMap::new();
        //注册
        insert_deserialized_enum_func!(map, IotDeviceColumn::DeviceType, DeviceType);
        insert_deserialized_enum_func!(map, HapAccessoryColumn::Category, BridgeCategory);
        insert_deserialized_enum_func!(map, HapBridgeColumn::Category, BridgeCategory);



        EnumFactory {
            map,
        }
    })
}


#[test]
fn test() {
    let mut active_model = IotDeviceActiveModel {
        ..Default::default()
    };

    // 通过string 反序列话成i32 的map
    let mut map: HashMap<ColumnEnumKey, DeserializedEnumFunc> = HashMap::new();

    let func1: DeserializedEnumFunc = |s| {
        let value: DeviceType = serde_json::from_str(format!("\"{}\"", s).as_mut_str())?;
        return Ok(value.into());
    };

    let a = func1("Normal").unwrap();
    active_model.set(IotDeviceColumn::DeviceType, a.into());
    println!("{:?}", active_model);
}