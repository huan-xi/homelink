use log::info;
use packed_struct::derive::PackedStruct;
use packed_struct::PackedStruct;
use aes::Aes128;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use serde_json::{Map, Value};

const HEADER_LEN: usize = 32;

#[derive(PackedStruct, Debug, Clone)]
#[packed_struct(endian = "msb")]
/// Struct describes protocol message header
pub struct MessageHeader {
    /// Always 0x2131
    pub magic_number: u16,
    /// Packet length including the header itself (32 bytes)
    pub packet_length: u16,
    /// Some unknown bytes
    pub unknown: u32,
    /// Device ID
    pub device_id: u32,
    /// Incrementing timestamp as reported by device
    pub stamp: u32,
    /// Checksum. See protocol description.
    pub checksum: [u8; 16],
}

impl Default for MessageHeader {
    fn default() -> Self {
        MessageHeader {
            magic_number: 0x2131,
            packet_length: 0,
            unknown: 0,
            device_id: 0,
            stamp: 0,
            checksum: [0; 16],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub header: MessageHeader,
    pub data: Vec<u8>,
}

pub trait RecvMessage {
    /// 获取返回的json 数据
    fn get_json_data(&self) -> &Map<String, Value>;
}

#[derive(Clone, Debug)]
pub struct JsonMessage {
    pub data: Map<String, Value>,
}

impl RecvMessage for JsonMessage {
    fn get_json_data(&self) -> &Map<String, Value> {
        &self.data
    }
}


impl Message {
    pub fn build(mut header: MessageHeader, data: Vec<u8>) -> Self {
        header.packet_length = (HEADER_LEN + data.len()) as u16;
        let mut msg = Self {
            header,
            data,
        };
        msg.checksum();
        msg
    }
    /// todo hello 包特殊处理
    pub fn checksum(&mut self) {
        let packet = self.pack_to_vec();
        let checksum = md5::compute(packet);
        self.header.checksum = *checksum;
    }
    pub fn parse(buf: &[u8]) -> anyhow::Result<Self> {
        let mut hdr: [u8; HEADER_LEN] = Default::default();
        hdr.copy_from_slice(&buf[..HEADER_LEN]);
        let header = MessageHeader::unpack(&hdr)?;
        info!("header:{:?}",header);
        let payload = &buf[HEADER_LEN..header.packet_length as usize];
        let data = payload.to_vec();
        Ok(Message {
            header,
            data,
        })
    }
    pub fn pack_to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.header.pack().unwrap());
        buf.extend_from_slice(&self.data);
        buf
    }

    pub fn unpack(token: &[u8; 16], buf: &[u8]) -> Message {
        let mut hdr: [u8; HEADER_LEN] = Default::default();
        hdr.copy_from_slice(&buf[..HEADER_LEN]);
        let header = MessageHeader::unpack(&hdr).unwrap();

        let payload = &buf[32..header.packet_length as usize];
        // log::info!("Got payload len={}: {:?}", payload.len(), payload);
        let data = Utils::decrypt(token, payload);
        info!("{}",String::from_utf8(data.to_vec()).unwrap());

        Message {
            header,
            data,
        }
    }
}


pub struct Utils {}

impl Utils {
    pub fn key_iv(token: &[u8; 16]) -> (Vec<u8>, Vec<u8>) {
        let key = md5::compute(token).to_vec();
        let mut iv_src = key.to_vec();
        iv_src.extend(token);
        let iv = md5::compute(iv_src).to_vec();
        (key, iv)
    }
    /// 解密
    pub fn decrypt(token: &[u8; 16], payload: &[u8]) -> Vec<u8> {
        if payload.is_empty() {
            return vec![];
        };
        let (key, iv) = Self::key_iv(token);
        let cipher = Cbc::<Aes128, Pkcs7>::new_from_slices(&key, &iv).unwrap();
        let mut buf = payload.to_vec();
        cipher.decrypt(&mut buf).unwrap().to_vec()
    }
    /// 加密
    pub fn encrypt(token: &[u8; 16], payload: &[u8]) -> Vec<u8> {
        let (key, iv) = Self::key_iv(token);
        let cipher = Cbc::<Aes128, Pkcs7>::new_from_slices(&key, &iv).unwrap();
        cipher.encrypt_vec(payload)
    }
}


