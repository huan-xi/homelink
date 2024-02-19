use std::array::TryFromSliceError;
use num_enum::TryFromPrimitive;
use packed_struct::derive::PackedStruct;
use serde::{Deserialize, Serialize};
use crate::BltResult;
use crate::error::BleError::UnpackError;



pub fn parse_xiaomi() {}




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