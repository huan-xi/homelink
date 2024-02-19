use std::iter::repeat;

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use crypto::digest::Digest;
use crypto::sha2::Sha512;
use rand::Rng;

use hap::Pin;

const BASE36: [char; 36] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J',
    'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'
];

pub struct HomeKitSetupUri {
    pub category: u64,
    pub password: u64,
    pub setup_id: String,
    pub version: u64,
    pub reserved: u64,
    // 2=IP, 4=BLE, 8=IP_WAC
    pub flags: u64,
}

impl Default for HomeKitSetupUri {
    fn default() -> Self {
        Self {
            /// 桥接方式接入
            category: 2,
            password: 0,
            setup_id: "".to_string(),
            version: 0,
            reserved: 0,
            flags: 2,
        }
    }
}

pub fn rand_mac_addr() -> hap::MacAddress {
    let mut rng = rand::thread_rng();
    let random_bytes: [u8; 6] = rng.gen();
    let mac_addr = hap::MacAddress::from(random_bytes);
    mac_addr
}

#[test]
pub fn test_rand_mac_addr() {
    println!("{}", rand_mac_addr());
    println!("{}", rand_mac_addr());
    println!("{}", rand_mac_addr());
}

pub fn rand_setup_id() -> String {
    let chars = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let max = chars.len();
    let mut setup_id = String::new();
    let mut rng = rand::thread_rng();

    for _ in 0..4 {
        let index = rng.gen_range(0, max);
        setup_id.push(chars.chars().nth(index).unwrap());
    }

    setup_id
}

pub fn compute_setup_hash(setup_id: &str, mac: &str) -> String {
    let mut hasher = Sha512::new();
    let input = format!("{}{}", setup_id, mac.to_uppercase());
    hasher.input_str(input.as_str());
    let len = hasher.output_bytes();
    let mut buf: Vec<u8> = repeat(0).take(len).collect();
    hasher.result(buf.as_mut());
    BASE64_STANDARD.encode(&buf[..4])
}

#[test]
pub fn test_compute_hash() {
    let setup_id = "F1PV";
    let mac = "5E:56:EA:F2:6E:7C";
    let hash = compute_setup_hash(setup_id, mac);
    println!("{}", hash);
    // assert_eq!("fXaN3w==", hash.as_str());
}

#[test]
pub fn test_setup_id() {
    println!("{}", rand_setup_id());
    println!("{}", rand_setup_id());
    println!("{}", rand_setup_id());
}

/// 随机生成pin码
pub fn rand_pin_code() -> u32 {
    let mut rng = rand::thread_rng();
    let invalid_numbers: [u32; 12] = [0, 11111111, 22222222, 33333333, 44444444, 55555555, 66666666, 77777777, 88888888, 99999999, 12345678, 87654321];

    let random_number = loop {
        let number = rng.gen_range(100_000_00, 999_999_99);
        if !invalid_numbers.contains(&number) {
            break number;
        }
    };
    random_number
}

#[test]
pub fn test_rand_pin_code() {
    println!("{}", rand_pin_code());
}

pub fn pin_code_from_str(pin: &str) -> Pin {
    let mut arr: [u8; 8] = [0; 8]; // 初始化一个长度为8的u8数组
    for (i, c) in pin.chars().enumerate() {
        if i < 8 {
            arr[i] = c.to_digit(10).unwrap() as u8
        }
    }
    Pin::new(arr).unwrap()
}


/// 生成setup uri
/// https://github.com/HomeSpan/HomeSpan/blob/master/docs/QRCodes.md
/// https://github.com/maximkulkin/esp-homekit/blob/0f3ef2ac2872ffe64dfe4e5d929420af327d48a5/tools/gen_qrcode#L18
pub fn gen_homekit_setup_uri(params: &HomeKitSetupUri) -> String {
    let mut payload: u64 = 0;
    payload |= (params.version & 0x7);

    payload <<= 4;
    payload |= (params.reserved & 0xf);

    payload <<= 8;
    payload |= (params.category & 0xff);

    payload <<= 4;
    payload |= (params.flags & 0xf);

    payload <<= 27;
    payload |= params.password & 0x7fffffff;

    let mut encoded_payload = String::new();
    for _ in 0..9 {
        encoded_payload.push(BASE36[(payload % 36) as usize]);
        payload /= 36;
    }

    format!("X-HM://{}{}", encoded_payload.chars().rev().collect::<String>(), params.setup_id)
}

#[test]
pub fn test2() {
    let bytes: [u8; 6] = [17, 231, 84, 31, 202, 172];
}

pub fn gen_homekit_setup_uri_default(pin_code: u64, category: u64, setup_id: String) -> String {
    gen_homekit_setup_uri(&HomeKitSetupUri {
        password: pin_code,
        setup_id,
        category,
        ..Default::default()
    })
}

#[test]
pub fn test_code() {
    let uri = gen_homekit_setup_uri(&HomeKitSetupUri {
        password: 29826556,
        setup_id: "3QYT".to_string(),
        category: 8,
        ..Default::default()
    });
    println!("{}", uri);
}

#[test]
pub fn test_output() {
    let uri = gen_homekit_setup_uri(&HomeKitSetupUri {
        password: 11122333,
        setup_id: "F1PV".to_string(),
        category: 2,
        ..Default::default()
    });
    // assert_eq!(uri, "X-HM://0081210CC3QYT");
    println!("{}", uri);
}
