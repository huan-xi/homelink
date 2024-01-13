use std::collections::HashMap;
use packed_struct::derive::PackedStruct;
use serde_json::Value;
use tap::Tap;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use anyhow::anyhow;
use futures_util::TryStreamExt;
use log::error;
use packed_struct::PackedStruct;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Eq, PartialEq, Clone, TryFromPrimitive, Hash,Serialize,Deserialize)]
#[repr(u64)]
pub enum BleValueType {
    /// 温度
    Action = 0x1001,
    Sleep = 0x1002,
    Temperature = 0x1004,
    Kettle = 0x1005,
    Humidity = 0x1006,
    Battery = 0x100a,
    ContactValue = 3,
}

impl BleValueType {
    pub fn unpack(&self, edata: Vec<u8>) -> anyhow::Result<serde_json::Value> {
        Ok(match self {
            BleValueType::Battery => {
                Value::Null
            }
            _ => {
                let bytes: [u8; 2] = edata.try_into()
                    .map_err(|e| anyhow!("数据转换错误:{:?}",e))?;
                ValueLsbI16::unpack(&bytes)?.into()
            }
        })
    }
}


#[derive(Default, Debug, Clone)]
pub struct BleValue {
    pub value_map: HashMap<BleValueType, Value>,
}

impl BleValue {
    pub fn extend(&mut self, other: Self) {
        self.value_map.extend(other.value_map);
    }
    pub fn set_value(&mut self, key: BleValueType, val: Value) {
        self.value_map.insert(key, val);
    }
}

impl BleValue {
    pub fn get_value(&self, value_type: BleValueType) -> Option<Value> {
        self.value_map.get(&value_type).map(|i| i.clone())
    }
}


///lsbI16 类型的值
#[derive(PackedStruct, Debug, Clone)]
#[packed_struct(endian = "lsb")]
pub struct ValueLsbI16 {
    pub value: i16,
}

impl Into<serde_json::Value> for ValueLsbI16 {
    fn into(self) -> Value {
        Value::from(self.value)
    }
}
