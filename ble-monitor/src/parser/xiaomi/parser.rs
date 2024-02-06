use impl_new::New;
use log::{error, info};
use tap::TapFallible;
use crate::BltResult;
use crate::error::BleError::{BleValueTypeError, NotSupported};
use crate::parse_advertisement::ServiceDataPacket;
use crate::parser::xiaomi::value_object::BleValueType;
use crate::parser::xiaomi_1::Packet;

#[derive(New)]
pub struct XiaomiParser;

impl XiaomiParser {
    pub fn parse(&self, data: &[u8]) -> BltResult<ServiceDataPacket> {
        let mut index = 5;
        let msg_length = data.len();
        let packet = Packet::unpack(data)?;
        if packet.ctrl.mesh {
            return Err(NotSupported("mesh"));
        }
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
        let payload_start = 0;
        let etype = payload[payload_start] as u16 | ((payload[payload_start + 1] as u16) << 8);
        let obj_length = payload[payload_start + 2];
        let next_start = payload_start + 3 + obj_length as usize;
        //todo 长度校验
        let edata = &payload[payload_start + 3..next_start];
        let tp = BleValueType::try_from(etype).map_err(|_| BleValueTypeError(etype))?;
        // println!("tp:{:?}", tp);
        let value = tp.unpack(edata).tap_err(|_| error!("小米payload 数据解码错误:etype:0x{:x},data:0x{:?}",etype,edata))?;
        info!("type:{:?}.value:{:?}",tp, value);
        Ok(ServiceDataPacket {
            etype,
            mac: packet.mac,
            edata: value,
        })
    }
}