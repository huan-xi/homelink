use std::array::TryFromSliceError;
use num_enum::TryFromPrimitive;
use packed_struct::derive::PackedStruct;
use serde::{Deserialize, Serialize};
use crate::BltResult;
use crate::error::BleError::UnpackError;

#[derive(Debug, Copy, Eq, PartialEq, Clone, TryFromPrimitive, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum XiaomiType {
    /// 水墨屏温湿度传感器
    LYWSD02 = 0x045B,
}

pub fn parse_xiaomi() {}

#[derive(Debug)]
pub struct Ctrl {
    pub(crate) mesh: bool,
    version: u16,
    auth_mode: u16,
    encrypted: bool,
    pub(crate) mac_include: bool,
    pub(crate) capability_include: bool,
}


#[derive(Debug)]
pub struct Packet {
    pub(crate) ctrl: Ctrl,
    /// 设备类型
    device_type: XiaomiType,
    /// 包id
    packet_id: u8,
    /// mac地址
    pub(crate) mac: [u8; 6],
    payload: Vec<u8>,
}

/// https://iot.mi.com/new/doc/accesses/direct-access/embedded-development/ble/object-definition
impl Packet {
    pub fn unpack(data: &[u8]) -> BltResult<Self> {
        let fr_ctrl = data[0] as u16 | (data[1] as u16) << 8;
        let ctrl = Ctrl {
            mesh: (fr_ctrl >> 7) & 1 == 1,
            version: fr_ctrl >> 12,
            auth_mode: (fr_ctrl >> 10) & 3,
            encrypted: fr_ctrl >> 3 & 1 == 1,
            mac_include: fr_ctrl >> 4 & 1 == 1,
            capability_include: ((fr_ctrl >> 5) & 1) == 1,
        };
        let device_id = data[2] as u16 | (data[3] as u16) << 8;
        let packet_id = data[4];
        let mut mac: [u8; 6] = data[5..11].try_into()
            .map_err(|e| UnpackError(Box::new(e)))?;
        mac.reverse();
        let device_type = XiaomiType::try_from(device_id)
            .map_err(|e| UnpackError(Box::new(e)))?;


        //todo packet id 去重
        //解码payload


        Ok(Self {
            ctrl,
            device_type,
            packet_id,
            mac,
            payload: vec![],
        })
    }
}


#[cfg(test)]
mod test {
    use crate::BltResult;
    use crate::error::BleError::BleValueTypeError;
    use crate::parser::xiaomi::value_object::BleValueType;
    use crate::parser::xiaomi_1::XiaomiType;

    #[test]
    pub fn test_parse_xiaomi() -> BltResult<()> {
        //0x045B
        /// 2248045b2070
        /// 22B9
        /// 17:75:BD:61:B9:22
        /// (70 20 5b 04 48) (22 b9 61bd7517)
        /// (09 0a (10 01) 19)
        ///
        ///   "70205b04d522b961bd7517 0409 1002 c4 00"
        ///
        /// 70205b044622b961bd751709061002ae01 湿度包
        ///                                  mac 地址         数据类型 数据长度  数据
        ///                     70205b0446(22b961bd7517) (09)(0610) (02) (ae01)
        let a = "70205b044622b961bd751709061002ae01";
        let data = hex::decode(a).unwrap();
        let mut index = 5;
        let msg_length = data.len();
        let packet = super::Packet::unpack(data.as_slice())?;
        if packet.ctrl.mac_include {
            index += 6;
        };

        if packet.ctrl.capability_include {
            index += 1;
            let capability_types = data[index - 1];
            if (capability_types & 0x20) != 0 {
                index += 1;
            }

            //容量检测
            // println!("capability_types:{}", capability_types);
        }

        let payload = &data[index..];
        // obj_typecode = payload[payload_start] + (payload[payload_start + 1] << 8)
        let payload_start = 0;
        let obj_typecode = payload[payload_start] as u16 | ((payload[payload_start + 1] as u16) << 8);
        // println!("obj_typecode:{:x}", obj_typecode);
        let obj_length = payload[payload_start + 2];
        // println!("obj_length:{:?}", obj_length);
        let next_start = payload_start + 3 + obj_length as usize;
        //todo 长度校验
        let edata = &payload[payload_start + 3..next_start];
        let tp = BleValueType::try_from(obj_typecode).map_err(|_| BleValueTypeError(obj_typecode))?;
        // println!("tp:{:?}", tp);
        let value = tp.unpack(edata)?;
        println!("type:{:?}.value:{:?}",tp, value);

        // println!("frctrl:{},frctrl_mesh:{frctrl_mesh}", frctrl);
        println!("payload len:{:?}", hex::encode(payload));
        println!("device_id{:?}", packet);
        ///9,15
        // println!("mac:{}", hex::encode(mac));
        //# check that data contains object
        /*    if frctrl_object_include != 0 {
                println!("object_include:{}", frctrl_object_include);
            }*/

        println!("xiaomi_1");
        Ok(())
    }
}