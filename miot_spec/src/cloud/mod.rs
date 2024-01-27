mod types;
pub mod state;
mod cookie_store_mutex;
mod mi_cloud_device_group;

use crypto::symmetriccipher::SynchronousStreamCipher;
use crypto::rc4::Rc4;
use base64;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::i64;
use std::io::Write;
use std::iter::repeat;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use anyhow::anyhow;
use base64::Engine;
use base64::engine::general_purpose;
use base64::engine::general_purpose::{STANDARD};
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use crypto::sha2::Sha256;
use hex::ToHex;
use log::{error, info};
use rand::distributions::Alphanumeric;
use rand::Rng;
use reqwest::{header, StatusCode, Url};
use reqwest::cookie::{Cookie, CookieStore};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::AsyncReadExt;
use crate::cloud::state::CookieState;
use anyhow::Result;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use reqwest::header::HeaderMap;
use tap::TapFallible;

pub struct Utils {}

mod test {
    use std::collections::{BTreeMap, HashMap};
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD;
    use base64::prelude::BASE64_STANDARD_NO_PAD;
    use log::info;
    use crate::cloud::Utils;

    #[test]
    pub fn test_nonce() {
        let a = Utils::gen_nonce();
        println!("a={:?}", a);
    }

    #[test]
    pub fn test_gen_nonce() {
        let a = Utils::gen_nonce();
        println!("a={:?}", a);
    }

    #[test]
    pub fn test_map() {
        let mut map = BTreeMap::new();
        map.insert("data".to_string(), "test".to_string());
        map.insert("data1".to_string(), "test".to_string());
        map.insert("data2".to_string(), "test".to_string());
        println!("map:{:?}", map);
    }

    #[test]
    pub fn test_gen_enc_signature() {
        let url = "https://api.io.mi.com/app/home/device_list";
        let method = "POST";
        let str = r#"{"getVirtualModel":true,"getHuamiDevices":1,"get_split_device":false,"support_smart_home":true}"#;
        let mut params = BTreeMap::new();
        params.insert("data".to_string(), str.to_string());
        let nonce = "W9afkMgX0qwBsaTy";
        let ssecurity = "mYxY+TqjOywvBtu5sSM/mw==";
        let signed_nonce = Utils::signed_nonce(ssecurity, nonce).unwrap();
        println!("signed_nonce={:?}", signed_nonce);
        let a = Utils::gen_enc_signature(url, method, signed_nonce.as_str(), &params).unwrap();
        Utils::generate_enc_params(url, method, signed_nonce.as_str(), nonce, &mut params, ssecurity).unwrap();
        info!("param:{:?}",params);
    }

    #[test]
    pub fn generate_enc_params() {
        let pl = "aAZcDBKHyBcAAAAAAbGj7A==";
        let mut map = BTreeMap::new();
        map.insert("test".to_string(), "test".to_string());
        let a = Utils::gen_enc_signature(
            "https://api.io.mi.com/app/home/device_list",
            "POST", pl, &map);
        println!("a={:?}", a);
        assert_eq!(a.unwrap(), "LcpJ0oYC+33hSUs+WAmb+c46GeY=");
    }

    #[test]
    pub fn signed_nonce() {
        let a = Utils::signed_nonce("test", "test");
        println!("a={:?}", a);
        assert_eq!(a.unwrap(), "EsOYiFaiTaaMnkmetdKmFb+pTYd7C0Ys1hzKX3uZWGw=");
    }

    #[test]
    pub fn test_encrypt_rc4() {
        let p = "test";
        let pl = "aAZcDBKHyBcAAAAAAbGj7A==";
        let a = pl.to_string();
        let text = hex::encode(a.as_bytes());
        println!("text={:?}", text);
        assert_eq!(text, "61415a6344424b4879426341414141414162476a37413d3d");

        let res = Utils::encrypt_rc4(p, pl).unwrap();
        assert_eq!(res, "0HjtuAI3Qr/R9BfGcGw8fGtfoWhcFDil");
    }

    #[test]
    fn test_base() {
        let hex = "d078edb8023742bfd1f417c6706c3c7c6b5fa1685c1438a5";
        let bytes = hex::decode(hex).unwrap();
        let b = STANDARD.encode(bytes.as_slice());
        println!("b={:?}", b);
        let c = BASE64_STANDARD_NO_PAD.encode(bytes.as_slice());
        println!("c={:?}", c);
        let d = BASE64_STANDARD_NO_PAD.encode(bytes.as_slice());
        println!("c={:?}", d);
    }

    #[test]
    pub fn test_rc4() {
        let p = "test";
        let pl = "aAZcDBKHyBcAAAAAAbGj7A==";
        let a = STANDARD.decode(p).unwrap();
        let a = hex::encode(a);
        let pl_hex = hex::encode(STANDARD.decode(pl).unwrap());
        println!("a={:?}", a);
        assert_eq!(a, "b5eb2d");
        assert_eq!(pl_hex, "68065c0c1287c8170000000001b1a3ec");

        let res = Utils::decrypt_rc4(p, pl).unwrap();
        let text = hex::encode(res);
        println!("text={:?}", text);
        assert_eq!(text, "d93febd754f2c1e0a8b67487309cded1");
    }
}


impl Utils {
    fn encrypt_rc4(password: &str, payload: &str) -> anyhow::Result<String> {
        let password = STANDARD.decode(password)?;
        let payload = payload.to_string();
        let len = payload.as_bytes().len();
        let mut encrypted_payload = Vec::new();
        encrypted_payload.resize(len, 0u8);
        let mut rc4 = Rc4::new(&password);
        rc4.process(&[0u8; 1024], &mut [0u8; 1024]);
        rc4.process(payload.as_bytes(), &mut encrypted_payload);
        let data = STANDARD.encode(encrypted_payload);
        Ok(data)
    }
    fn decrypt_rc4(password: &str, payload: &str) -> anyhow::Result<Vec<u8>> {
        let password = STANDARD.decode(password)?;
        let payload = STANDARD.decode(payload)?;
        let len = payload.len();
        let mut encrypted_payload = Vec::new();
        encrypted_payload.resize(len, 0u8);
        let mut rc4 = Rc4::new(&password);
        rc4.process(&[0u8; 1024], &mut [0u8; 1024]);
        rc4.process(payload.as_slice(), &mut encrypted_payload);
        Ok(encrypted_payload)
    }
    fn generate_enc_params(url: &str, method: &str, signed_nonce: &str, nonce: &str, params: &mut BTreeMap<String, String>, ssecurity: &str) -> Result<()> {
        let sign = Self::gen_enc_signature(url, method, signed_nonce, &params)?;
        params.insert("rc4_hash__".to_string(), sign);
        for (_, v) in params.iter_mut() {
            *v = Self::encrypt_rc4(signed_nonce, v.as_str())?;
        }
        let sign = Self::gen_enc_signature(url, method, signed_nonce, &params)?;
        params.insert("signature".to_string(), sign);
        params.insert("ssecurity".to_string(), ssecurity.to_string());
        params.insert("_nonce".to_string(), nonce.to_string());
        Ok(())
    }
    fn gen_enc_signature(url: &str, method: &str, signed_nonce: &str, params: &BTreeMap<String, String>) -> Result<String> {
        let mut signature_params = vec![
            method.to_uppercase(),
            url.split("com").nth(1).ok_or_else(|| anyhow!("Invalid URL"))?.replace("/app/", "/"),
        ];
        for (k, v) in params.iter() {
            signature_params.push(format!("{}={}", k, v));
        }
        signature_params.push(signed_nonce.to_owned());
        let signature_string = signature_params.join("&");
        // let str = r#"POST&/home/device_list&data={"getVirtualModel":true,"getHuamiDevices":1,"get_split_device":false,"support_smart_home":true}&l2YPQo7fjEnBKzWSvNXtPn3hnhBR5uJ65/ZMqpWkMQQ="#;
        // assert_eq!(signature_string.as_str(), str);
        let mut hasher = Sha1::new();
        hasher.input_str(signature_string.as_str());

        let mut buf: Vec<u8> = repeat(0).take((hasher.output_bits() + 7) / 8).collect();
        hasher.result(&mut buf);
        // hasher.result(hashed_signature.as_mut_slice());
        let encoded_signature = STANDARD.encode(buf.to_vec());
        Ok(encoded_signature)
    }
    fn signed_nonce(ssecret: &str, nonce: &str) -> Result<String> {
        let mut hasher = Sha256::new();
        let ssecret = STANDARD.decode(ssecret.as_bytes())
            .map_err(|e| anyhow!("Failed to decode ssecret: {}", e))?;
        let nonce = STANDARD.decode(nonce.as_bytes())
            .map_err(|e| anyhow!("Failed to decode nonce: {}", e))?;
        hasher.input(ssecret.as_slice());
        hasher.input(nonce.as_slice());
        let mut buf: Vec<u8> = repeat(0).take((hasher.output_bits() + 7) / 8).collect();
        hasher.result(buf.as_mut_slice());
        Ok(STANDARD.encode(buf.as_slice()))
    }

    fn gen_nonce() -> anyhow::Result<String> {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow!("Time went backwards"))?
            .as_millis() as i64;

        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 8] = rng.gen();
        let random_int = i64::from_be_bytes(random_bytes).overflowing_sub(1 << 63).0;

        let part2 = millis as u64 / 60000;
        let mut nonce_bytes = random_int.to_be_bytes().to_vec();
        let mut part2_bytes = part2.to_be_bytes().to_vec();
        nonce_bytes.extend_from_slice(&part2_bytes[4..8]);
        Ok(STANDARD.encode(&nonce_bytes))
    }

    fn get_random_agent_id() -> String {
        let mut rng = rand::thread_rng();
        let result_str: String = (0..13)
            .map(|_| "ABCDEF".chars().nth(rng.gen_range(0..6)).unwrap())
            .collect();
        result_str
    }

    fn get_random_string(length: usize) -> String {
        let result_str: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect();
        result_str
    }
}


const JSON_START: &'static str = "&&&START&&&";

#[derive(Clone, Serialize, Default, Deserialize)]
pub struct Info {
    ssecurity: String,
    service_token: String,
    user_id: u64,
    cuser_id: String,
    pass_token: String,
}

pub struct MiCloud {
    client: reqwest::Client,
    state: CookieState,
    info_path: PathBuf,
    info: Option<Info>,
    country: String,
    username: String,
    password: Option<String>,
}

unsafe impl Send for MiCloud {}

unsafe impl Sync for MiCloud {}


impl MiCloud {
    pub async fn new(session_path: &str, username: String, password: Option<String>) -> anyhow::Result<Self> {
        let agent_id = Utils::get_random_agent_id();
        let useragent = format!("Android-7.1.1-1.0.0-ONEPLUS A3010-136-{} APP/xiaomi.smarthome APPV/62830", agent_id);
        let mut headers = header::HeaderMap::new();
        headers.insert(header::USER_AGENT, header::HeaderValue::from_str(useragent.as_str()).unwrap());
        headers.insert("Connection", header::HeaderValue::from_str("keep-alive").unwrap());
        let path_id = hex::encode(md5::compute(username.as_str()).to_vec());
        let cookie_path = format!("{}/cookie_{}.json", session_path, path_id);
        let info_path = PathBuf::from(format!("{}/info_{}.json", session_path, path_id));
        let info = match tokio::fs::File::open(&info_path).await {
            Ok(mut f) => {
                let mut contents = String::new();
                f.read_to_string(&mut contents).await?;
                let info: serde_json::Result<Info> = serde_json::from_str(contents.as_str());
                info.ok()
            }
            Err(_) => None
        };
        let state = CookieState::try_new(PathBuf::from(cookie_path))?;
        if info.is_none() {
            state.cookie_store.lock().unwrap().clear();
        };

        let client = reqwest::ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .default_headers(
                headers
            )
            .cookie_store(true)
            .http1_title_case_headers()
            // .proxy(reqwest::Proxy::all("http://127.0.0.1:7890")?)
            .cookie_provider(state.cookie_store.clone())
            .build()
            .unwrap();
        //登入状态?

        Ok(Self {
            client,
            state,
            info_path,
            info,
            country: "cn".to_string(),
            username,
            password,
        })
    }
    pub async fn save_info(&self) -> anyhow::Result<()> {
        if let Some(info) = self.info.clone() {
            let info = serde_json::to_string(&info)?;
            //创建文件夹
            let path = self.info_path.parent().unwrap();
            tokio::fs::create_dir_all(path).await?;
            let mut file = File::create(self.info_path.as_path())?;
            file.write_all(info.as_bytes())?;
        }
        Ok(())
    }
    pub async fn login(&mut self) -> anyhow::Result<()> {
        let sign = self.login_step1().await?;
        info!("start login step1");
        self.info = None;
        self.login_step2(sign.as_str()).await?;
        let has_info = match self.info.as_ref() {
            None => { false }
            Some(i) => !i.service_token.is_empty()
        };

        let location = if !sign.starts_with("http") || !has_info {
            info!("start login step2");
            self.login_step2(sign.as_str()).await?
        } else {
            sign
        };
        info!("start login step3");
        let status = self.login_step3(location.as_str()).await?;
        if status == StatusCode::OK {
            return Ok(());
        }
        // 403 错误
        Err(anyhow!("登录米家账号失败"))
    }

    pub fn get_info(&self) -> anyhow::Result<Info> {
        match self.info.clone() {
            None => {
                return Err(anyhow!("未登录"));
            }
            Some(i) => Ok(i)
        }
    }
    pub async fn get_devices(&self) -> anyhow::Result<serde_json::Value> {
        let str = r#"{"getVirtualModel":true,"getHuamiDevices":1,"get_split_device":false,"support_smart_home":true}"#;
        self.call_api("/home/device_list", str).await
    }
    pub async fn call_api(&self, url: &str, param_str: &str) -> anyhow::Result<serde_json::Value> {
        let api_url = self._get_api_url(url);
        let nonce = Utils::gen_nonce()?;
        let info = self.get_info()?;
        let mut params = BTreeMap::new();
        params.insert("data".to_string(), param_str.to_string());
        let ssecurity = info.ssecurity;
        if info.service_token.is_empty() {
            return Err(anyhow!("未登录"));
        };
        let signed_nonce = Utils::signed_nonce(ssecurity.as_str(), nonce.as_str())?;

        Utils::generate_enc_params(api_url.as_str(), "POST",
                                   signed_nonce.as_str(), nonce.as_str(),
                                   &mut params,
                                   ssecurity.as_str())?;

        let mut header = HeaderMap::new();

        header.insert("Accept-Encoding", "identity".parse()?);
        header.insert("x-xiaomi-protocal-flag-cli", "PROTOCAL-HTTP2".parse()?);
        header.insert("content-type", "application/x-www-form-urlencoded".parse()?);
        header.insert("MIOT-ENCRYPT-ALGORITHM", "ENCRYPT-RC4".parse()?);
        // 设置cookies
        {
            let mut cookies = self.state.cookie_store.lock().unwrap();

            let url = Url::parse(api_url.as_str())?;
            let ck = reqwest_cookie_store::RawCookie::new("userId", info.user_id
                .to_string());
            cookies.insert_raw(&ck, &url)?;
            let token = info.service_token;

            let ck = reqwest_cookie_store::RawCookie::new("yetAnotherServiceToken", token.clone());
            cookies.insert_raw(&ck, &url)?;
            let ck = reqwest_cookie_store::RawCookie::new("serviceToken", token.clone());
            cookies.insert_raw(&ck, &url)?;
            let ck = reqwest_cookie_store::RawCookie::new("is_daylight", "0");
            cookies.insert_raw(&ck, &url)?;
            let ck = reqwest_cookie_store::RawCookie::new("dst_offset", "0");
            cookies.insert_raw(&ck, &url)?;
            let ck = reqwest_cookie_store::RawCookie::new("locale", "zh_CN");
            cookies.insert_raw(&ck, &url)?;
            let ck = reqwest_cookie_store::RawCookie::new("timezone", "GMT+08:00");
            cookies.insert_raw(&ck, &url)?;
            let ck = reqwest_cookie_store::RawCookie::new("channel", "MI_APP_STORE");
            cookies.insert_raw(&ck, &url)?;
            let ck = reqwest_cookie_store::RawCookie::new("deviceId", "gnjplb");
            cookies.insert_raw(&ck, &url)?;
            let ck = reqwest_cookie_store::RawCookie::new("sdkVersion", "3.8.6");
            cookies.insert_raw(&ck, &url)?;
        }

        //url编码
        info!("params:{:?}", params);

        let resp = self.client
            .post(api_url)
            .headers(header)
            //提交表单数据
            .form(&params)
            .send().await?;
        let text = resp.text().await?;
        // info!("resp:{}", text.as_str());
        if text.starts_with("{\"") {
            error!("调用失败返回值:{}",text);
            return Err(anyhow!("登录失败 请重新登入"));
        }
        let sign = Utils::signed_nonce(ssecurity.as_str(), nonce.as_str())?;
        let bytes = Utils::decrypt_rc4(sign.as_str(), text.as_str())?;
        // let text = String::from_utf8(bytes)?;

        let text = String::from_utf8(bytes)?;
        // 写入json
        // let mut file = File::create("./data/mi_devices.json")?;
        // file.write_all(text.as_bytes())?;

        let val: serde_json::Value = serde_json::from_str(text.as_str())?;


        return Ok(val);
    }
    fn _get_api_url(&self, path: &str) -> String {
        let country_lower = self.country.as_str().trim().to_lowercase();
        let api_url = if country_lower == "cn" {
            "".to_string()
        } else {
            format!("{}.", country_lower)
        };
        format!("https://{}api.io.mi.com/app{}", api_url, path)
    }


    async fn login_step3(&mut self, location: &str) -> anyhow::Result<StatusCode> {
        let resp = self.client.get(location).send().await?;
        //userId=1254140309;
        let cookies: Option<String> = self.state.cookie_store
            .lock().unwrap()
            .get("sts.api.io.mi.com", "/", "serviceToken")
            .and_then(|f| Option::from(f.value().to_owned()));
        match cookies {
            None => {
                return Err(anyhow!("获取cookie失败"));
            }
            Some(s) => {
                info!("service_token:{}", s.as_str());
                self.info.as_mut().unwrap().service_token = s;
                self.save_info().await?;
            }
        }
        Ok(resp.status())
    }

    /// 获取sign or location
    async fn login_step1(&self) -> anyhow::Result<String> {
        let url = Url::parse("https://account.xiaomi.com/pass/serviceLogin?sid=xiaomiio&_json=true")?;
        let resp = self.client.get(url).send().await?;
        let text = resp.text().await?;
        println!("{}", text);
        let text = text.as_str().replace(JSON_START, "");
        //转成json
        let mut json_map: serde_json::map::Map<String, Value> = serde_json::from_str(text.as_str()).unwrap();
        let sign = json_map.remove("_sign");
        match sign {
            None => {}
            Some(s) => {
                // self.state.save()?;
                return Ok(s.as_str().unwrap().to_string());
            }
        }
        if let Some(location) = json_map.remove("location") {
            if let Some(val) = location.as_str() {
                return Ok(val.to_string());
            }
        };

        Err(anyhow::anyhow!("获取sign 失败"))
    }

    async fn login_step2(&mut self, sign: &str) -> anyhow::Result<String> {
        let url = "https://account.xiaomi.com/pass/serviceLoginAuth2";
        let hash: String = md5::compute(self.password.as_ref()
            .ok_or(anyhow!("未设置密码"))?
            .as_bytes()).encode_hex_upper();
        let mut params = BTreeMap::new();
        params.insert("sid", "xiaomiio");
        params.insert("baz", "quux");
        params.insert("callback", "https://sts.api.io.mi.com/sts");
        params.insert("user", self.username.as_str());
        params.insert("_json", "true");
        params.insert("qs", "%3Fsid%3Dxiaomiio%26_json%3Dtrue");
        params.insert("_sign", sign);
        params.insert("hash", hash.as_str());
        // println!("params:{}", post_data.to_string());
        let resp = self.client.post(url)
            .form(&params)
            .send()
            .await.unwrap();
        let value = resp
            .headers()
            .get("Set-Cookie")
            .ok_or(anyhow!("获取Set-Cookie失败"))?;
        info!("value:{:?}", value);
        info!("value:{:?}", value.to_str());


        let text = resp.text().await?;
        if !text.starts_with(JSON_START) {
            return Err(anyhow::anyhow!("登录失败 请检查用户名密码"));
        };
        let text = text.replace(JSON_START, "");
        //转成json
        let mut json_map: serde_json::map::Map<String, Value> = serde_json::from_str(text.as_str())
            .map_err(|e| anyhow!("json解析失败"))?;

        let code = json_map.remove("code")
            .and_then(|i| i.as_u64());
        if code != Some(0) {
            let description = json_map.remove("description")
                .and_then(|f| f.as_str().map(|i| i.to_string()))
                .unwrap_or("未知原因".to_string());
            return Err(anyhow!("登录失败,错误信息:{}",description));
        }

        if let Some(location) = json_map.remove("location") {
            if let Some(l) = location.as_str() {
                let ssecurity = json_map.remove("ssecurity")
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .ok_or(anyhow!("获取ssecurity 失败"))
                    .tap_err(|e| {
                        //{"qs":"%3Fsid%3Dxiaomiio%26_json%3Dtrue","code":70016,"description"
                        error!("登录第二步失败,text:{}",text.as_str())
                    })?;

                let user_id = json_map.remove("userId")
                    .and_then(|v| v.as_u64())
                    .ok_or(anyhow!("获取user_id失败"))?;
                let cuser_id = json_map.remove("cUserId")
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .ok_or(anyhow!("获取cuser_id失败"))?;
                let pass_token = json_map.remove("passToken")
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .ok_or(anyhow!("获取pass_token失败"))?;


                let info = Info {
                    ssecurity: ssecurity.to_string(),
                    service_token: "".to_string(),
                    user_id,
                    cuser_id: cuser_id.to_string(),
                    pass_token: pass_token.to_string(),
                };
                self.info = Some(info.clone());
                self.save_info().await?;


                return Ok(l.to_string());
            }
        }
        Err(anyhow::anyhow!("获取location 失败"))
    }
}


