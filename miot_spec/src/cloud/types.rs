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

#[test]
fn test() {
    let json_str = r#"{
        "did":"1023054714",
        "token":"5f06bdf8555acc4ea0b2199b54c0e742",
        "longitude":"0.0",
        "latitude":"0.0",
        "name":"\u5438\u9876\u706f",
        "pid":"16",
        "localip":"",
        "mac":"74:A3:4A:E1:F9:E2",
        "ssid":"",
        "bssid":"",
        "parent_id":"",
        "parent_model":"",
        "show_mode":1,
        "model":"zimi.switch.dhkg01",
        "adminFlag":1,
        "shareFlag":0,
        "permitLevel":16,
        "isOnline":true,
        "desc":"\u8bbe\u5907\u5728\u7ebf ",
        "extra":{
            "isSetPincode":0,
            "pincodeType":0,
            "fw_version":"2.1.1_0117",
            "needVerifyCode":0,
            "isPasswordEncrypt":0
        },
        "uid":1254140309,
        "pd_id":1945,
        "password":"",
        "p2p_id":"",
        "rssi":0,
        "family_id":0,
        "reset_flag":0
    }"#;

    let data: Data = serde_json::from_str(json_str).unwrap();
    // 转
    println!("{:#?}", data);
}