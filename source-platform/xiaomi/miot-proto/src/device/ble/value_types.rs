use std::collections::HashMap;
use packed_struct::derive::PackedStruct;
use serde_json::Value;

use num_enum::TryFromPrimitive;

use anyhow::anyhow;
use futures_util::TryStreamExt;

use packed_struct::PackedStruct;
use serde::{Deserialize, Serialize};
use xiaomi_ble_packet::ble_value_type::{BleValue, MiBleValueType};


#[derive(Default, Debug, Clone)]
pub struct BleValues {
    pub value_map: HashMap<MiBleValueType, BleValue>,
}

impl BleValues {
    pub fn extend(&mut self, other: Self) {
        self.value_map.extend(other.value_map);
    }
    pub fn set_value(&mut self, key: MiBleValueType, val:  BleValue) {
        self.value_map.insert(key, val);
    }
}

impl BleValues {
    pub fn get_value(&self, value_type: MiBleValueType) -> Option<BleValue> {
        self.value_map.get(&value_type).cloned()
    }
}


///lsbI16 类型的值
#[derive(PackedStruct, Debug, Clone)]
#[packed_struct(endian = "lsb")]
pub struct ValueLsbI16 {
    pub value: i16,
}

impl From<ValueLsbI16> for serde_json::Value {
    fn from(val: ValueLsbI16) -> Self {
        Value::from(val.value)
    }
}
