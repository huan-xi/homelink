use std::collections::HashMap;
use impl_new::New;
use log::{debug, error, info};
use tap::TapFallible;
use xiaomi_ble_packet::ble_value_type::MiBleValueType;
use crate::BltResult;
use crate::error::BleError::{BleValueTypeError, NotSupported, UnpackDataError};
use crate::parse_advertisement::ServiceDataPacket;
use crate::parser::xiaomi::packet::Packet;

#[derive(New)]
pub struct XiaomiParser {
    aes_keys: HashMap<[u8; 6], String>,
}

impl XiaomiParser {
    pub fn parse(&self, data: &[u8]) -> BltResult<Option<ServiceDataPacket>> {
        let mut index = 5;
        let packet = Packet::unpack(data)?;
        if packet.ctrl.mesh {
            let a = packet.mac;
            debug!("不支持mesh设备: {:?}",a);
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
        if !packet.ctrl.object_include {
            return Ok(None);
        };
        if packet.ctrl.encrypted {
            if packet.ctrl.version <= 3 {
                todo!("解密数据v3")
            } else {
                self.decrypt_mibeacon_v4_v5(&packet, index, data)?;
            }
        }


        let payload = &data[index..];
        let payload_length = payload.len();

        let payload_start = 0;
        // assume that the data may have several values of different types
        // while payload_length >= payload_start + 3 {}
        // 确保数据大于3,etype 2字节,obj_length 1字节
        if payload_length < payload_start + 3 {
            error!("小米payload 长度错误:0x{:?}", payload);
            return Err(BleValueTypeError(0));
        }

        let etype = payload[payload_start] as u16 | ((payload[payload_start + 1] as u16) << 8);
        let obj_length = payload[payload_start + 2];
        let next_start = payload_start + 3 + obj_length as usize;
        // 长度校验
        if payload_length < next_start {
            error!("小米payload 长度错误:etype:0x{:x},data:0x{:?}",etype,payload);
            return Err(BleValueTypeError(etype));
        };


        let edata = &payload[payload_start + 3..next_start];
        let tp = MiBleValueType::try_from(etype).map_err(|_| BleValueTypeError(etype))?;
        // println!("tp:{:?}", tp);
        let value = tp.unpack(edata)
            .tap_err(|_| error!("小米payload 数据解码错误:etype:0x{:x},data:0x{:?}",etype,edata))?;
        info!("type:{:?}.value:{:?}",tp, value);
        Ok(Some(ServiceDataPacket {
            etype,
            mac: packet.mac.unwrap_or([0; 6]),
            edata: edata.to_vec(),
        }))
    }
    fn decrypt_mibeacon_v4_v5(&self, _packet: &Packet, index: usize, data: &[u8]) -> BltResult<Vec<u8>> {
        if data.len() < index + 9 {
            return Err(UnpackDataError("数据长度错误"));
        };
        todo!();
        // self.aes_keys.get(&packet.mac)
        //     .ok_or(UnpackDataError("aes key not found"))
        //     .tap_err(|_| debug!("mac:{},aes key not found",hex::encode(packet.mac)))?;
        //

        Ok(vec![])
    }
}


#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::env;
    use log::error;
    

    #[test]
    pub fn test1() {
        env::set_var("RUST_LOG", "debug");
        pretty_env_logger::init();
        /// mesh 包  11:3a
        let hex_str = "4859f607c85e659dcd11bf348a060007244f59";
        // let hex_str = "0541393152(368106ac8226)e6e03e";
        // NotSupportedDeviceType(5381)
        // let hex_str = "552b051500(0511011a0510)a305";
        // let hex_str = "552b051500(0511011a0510)a305";
        //  54:EF:44:E4:E6:1E
        // "54:EF:44:E4:FF:F9"
        let _aes_key = "";
        let data = hex::decode(hex_str).unwrap();
        let map = HashMap::new();
        let packet = super::XiaomiParser::new(map)
            .parse(data.as_slice());
        println!("{:?}", packet);
        match packet {
            Ok(_) => {}
            Err(e) => {
                error!("error: {}", e);
            }
        }
    }
}