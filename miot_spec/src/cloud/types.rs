use serde::{Deserialize};

#[derive(Debug, Deserialize)]
struct Data {
    did: Option<String>,
    token: Option<String>,
    longitude: Option<String>,
    latitude: Option<String>,
    name: Option<String>,
    pid: Option<String>,
    localip: Option<String>,
    mac: Option<String>,
    ssid: Option<String>,
    bssid: Option<String>,
    parent_id: Option<String>,
    parent_model: Option<String>,
    show_mode: Option<i32>,
    model: Option<String>,
    adminFlag: Option<i32>,
    shareFlag: Option<i32>,
    permitLevel: Option<i32>,
    isOnline: Option<bool>,
    desc: Option<String>,
    extra: Option<Extra>,
    uid: Option<u64>,
    pd_id: Option<u64>,
    password: Option<String>,
    p2p_id: Option<String>,
    rssi: Option<i32>,
    family_id: Option<i32>,
    reset_flag: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct Extra {
    isSetPincode: Option<i32>,
    pincodeType: Option<i32>,
    fw_version: Option<String>,
    needVerifyCode: Option<i32>,
    isPasswordEncrypt: Option<i32>,
}

