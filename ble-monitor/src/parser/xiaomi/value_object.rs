use packed_struct::derive::PackedStruct;

use num_enum::TryFromPrimitive;
use futures_util::TryStreamExt;
use packed_struct::PackedStruct;
use serde::{Deserialize, Serialize};
use crate::{BleValue, BltResult};
use crate::error::BleError::UnpackDataError;

#[derive(Debug, Copy, Eq, PartialEq, Clone, TryFromPrimitive, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum BleValueType {
    Action = 0x1001,
    Sleep = 0x1002,
    Temperature = 0x1004,
    Kettle = 0x1005,
    Humidity = 0x1006,
    Battery = 0x100a,
    ContactValue = 3,
}

impl BleValueType {
    pub fn unpack(&self, edata: &[u8]) -> BltResult<BleValue> {
        Ok(match self {
            BleValueType::Battery => {
                BleValue::U8(edata[0])
            }
            _ => {
                let bytes: [u8; 2] = edata.try_into()
                    .map_err(|e| UnpackDataError("数据转换错误:{e}"))?;
                BleValue::I16(ValueLsbI16::unpack(&bytes)?.value)
            }
        })
    }
}

///lsbI16 类型的值
#[derive(PackedStruct, Debug, Clone)]
#[packed_struct(endian = "lsb")]
pub struct ValueLsbI16 {
    pub value: i16,
}
