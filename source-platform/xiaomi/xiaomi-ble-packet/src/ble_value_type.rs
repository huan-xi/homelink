use num_enum::{IntoPrimitive, TryFromPrimitive};
use packed_struct::derive::PackedStruct;
use packed_struct::PackedStruct;
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;
use crate::error::MiPacketError::UnpackDataError;
use crate::MiPacketResult;
#[derive(Debug,Clone)]
pub enum BleValue {
    U8(u8),
    I16(i16),
}

impl BleValue{
    pub fn as_u64(&self) -> u64 {
        match self {
            BleValue::U8(v) => *v as u64,
            BleValue::I16(v) => *v as u64,
        }
    }
}


/// 米家蓝牙值类型
#[derive(Debug, EnumString, Copy, Eq, PartialEq, Clone, TryFromPrimitive,IntoPrimitive, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum MiBleValueType {
    Action = 0x1001,
    Sleep = 0x1002,
    Temperature = 0x1004,
    Kettle = 0x1005,
    Humidity = 0x1006,
    Battery = 0x100a,
    ContactValue = 3,
}



///lsbI16 类型的值
#[derive(PackedStruct, Debug, Clone)]
#[packed_struct(endian = "lsb")]
pub struct ValueLsbI16 {
    pub value: i16,
}

impl MiBleValueType {
    pub fn unpack(&self, edata: &[u8]) -> MiPacketResult<BleValue> {
        Ok(match self {
            MiBleValueType::Battery => {
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
