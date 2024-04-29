use std::collections::HashMap;

use num_enum::{TryFromPrimitive};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{BltResult};
use crate::error::BleError::NotSupportedPlatform;
use crate::parser::xiaomi::parser::XiaomiParser;


#[derive(Debug, Copy, Eq, PartialEq, Clone, TryFromPrimitive, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum BlePlatform {
    /// 小米
    Xiaomi = 0xfe95,
}


#[derive(Debug, Clone)]
pub struct ServiceDataPacket {
    pub mac: [u8; 6],
    pub etype: u16,
    pub edata: Vec<u8>,
}


/// 解析数据包
pub fn parse_advertisement(uuid: &Uuid, data: &[u8]) -> BltResult<Option<ServiceDataPacket>> {
    let uuid_128:&[u8] = uuid.as_ref();
    let uuid_16 = (uuid_128[2] as u16) << 8 | uuid_128[3] as u16;
    match BlePlatform::try_from(uuid_16) {
        Ok(platform) => {
            match platform {
                BlePlatform::Xiaomi => {
                    XiaomiParser::new(HashMap::new()).parse(data)
                }
            }
        }
        Err(_) => {
            Err(NotSupportedPlatform(uuid_16))
        }
    }

}

#[cfg(test)]
mod test {
    use std::str::FromStr;
    use std::u16;
    
    

    #[test]
    fn test_parse_advertisement() {
        let uuid = uuid::Uuid::from_str("0000fe95-0000-1000-8000-00805f9b34fb").unwrap();
        let uuid_128 = uuid.as_ref();
        let uuid_16 = (uuid_128[2] as u16) << 8 | uuid_128[3] as u16;
        match uuid_16 {
            0xfe95 => {
                println!("UUID-16: {:04x}", uuid_16);
            }
            _ => {}
        };
        // let uuid_16 = u16::from_be_bytes(uuid.as_bytes().as_slice());

        println!("UUID-16: {:04x}", uuid_16);
    }
}