use serde::Deserialize;

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
    #[serde(rename = "camelCase")]
    admin_flag: Option<i32>,
    #[serde(rename = "camelCase")]
    share_flag: Option<i32>,
    #[serde(rename = "camelCase")]
    permit_level: Option<i32>,
    #[serde(rename = "camelCase")]
    is_online: Option<bool>,
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
    #[serde(rename = "camelCase")]
    is_set_pincode: Option<i32>,
    #[serde(rename = "camelCase")]
    pincode_type: Option<i32>,

    fw_version: Option<String>,
    #[serde(rename = "camelCase")]
    need_verify_code: Option<i32>,
    #[serde(rename = "camelCase")]
    is_password_encrypt: Option<i32>,
}

