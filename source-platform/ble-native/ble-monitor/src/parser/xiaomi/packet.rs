use log::debug;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use tap::TapFallible;
use crate::BltResult;
use crate::error::BleError::{NotSupportedDeviceType, PacketUnpackError};

#[derive(Debug, Copy, Eq, PartialEq, Clone, TryFromPrimitive, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum XiaomiType {
    /// 水墨屏温湿度传感器
    LYWSD02 = 0x045B,
    /// 人体传感器2 lumi.motion.bmgl01
    RTCGQ02LM = 0x0A8D,
    ///t500
    M1sT500 = 0x0489,
    /// 米家夜灯 "yeelink.light.nl1"
    MJYD02YL = 0x07F6,
    //0x2809
    Unknown1 = 0x799,
    Unknown2 = 0x2809,
}

#[derive(Debug)]
pub struct Ctrl {
    pub(crate) mesh: bool,
    pub(crate) version: u16,
    auth_mode: u16,
    pub(crate) encrypted: bool,
    pub(crate) mac_include: bool,
    pub(crate) capability_include: bool,
    pub(crate) object_include: bool,
}

#[derive(Debug)]
pub struct Packet {
    pub(crate) ctrl: Ctrl,
    /// 设备类型
    device_type: XiaomiType,
    /// 包id
    packet_id: u8,
    /// mac地址
    pub(crate) mac: Option<[u8; 6]>,
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
            object_include: ((fr_ctrl >> 6) & 1) == 1,
        };
        let device_id = data[2] as u16 | (data[3] as u16) << 8;
        let packet_id = data[4];
        let mac = if ctrl.mac_include {
            let mut mac: [u8; 6] = data[5..11].try_into()
                .map_err(|e| PacketUnpackError(Box::new(e)))?;
            mac.reverse();
            Some(mac)
        } else {
            None
        };
        let device_type = XiaomiType::try_from(device_id)
            .map_err(|_e| NotSupportedDeviceType(device_id))
            .tap_err(|_e| debug!("不支持设备类型,did:{device_id},mac:{:?}",mac))?;


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

