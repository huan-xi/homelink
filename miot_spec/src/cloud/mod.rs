mod types;
pub mod state;
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use anyhow::anyhow;
use hex::ToHex;
use log::info;
use rand::distributions::Alphanumeric;
use rand::Rng;
use reqwest::{header, StatusCode};
use reqwest::cookie::CookieStore;
use serde_json::ser::Compound::Map;
use serde_json::{json, Value};
use serde_json::Value::Object;
use crate::cloud::state::State;


pub struct Utils {}

impl Utils {

    fn signed_nonce(ssecret: &str, nonce: &str) -> String {
        let mut m = Sha256::new();
        m.update(&base64::decode(ssecret).unwrap());
        m.update(&base64::decode(nonce).unwrap());

        let digest = m.finalize();

        let encoded_digest = base64::encode(&digest);
        encoded_digest
    }
    fn gen_nonce() -> String {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        let mut rng = rand::thread_rng();
        let random_num: i64 = rng.gen_range(0..(2i64.pow(64) - 2i64.pow(63)));

        let mut nonce_bytes = (random_num - 2i64.pow(63)).to_be_bytes().to_vec();

        let part2 = millis / 60000;
        nonce_bytes.extend_from_slice(&(part2.to_be_bytes()));

        let encoded_nonce = base64::encode(&nonce_bytes);
        encoded_nonce
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


pub struct MiCloud {
    pub client: reqwest::Client,
    pub username: String,
    pub password: String,
}


impl MiCloud {
    pub fn new(session_path: &str, username: String, password: String) -> anyhow::Result<Self> {
        let agent_id = Utils::get_random_agent_id();
        let useragent = format!("Android-7.1.1-1.0.0-ONEPLUS A3010-136-{} APP/xiaomi.smarthome APPV/62830", agent_id);
        let mut headers = header::HeaderMap::new();
        headers.insert(header::USER_AGENT, header::HeaderValue::from_str(useragent.as_str()).unwrap());
        let cookie_path = format!("{}/cookie.json_2", session_path);
        let state = State::try_new(PathBuf::from(cookie_path))?;
        let client = reqwest::ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .default_headers(
                headers
            )
            .cookie_store(true)
            .cookie_provider(state.cookie_store.clone())
            .build()
            .unwrap();
        //登入状态?

        Ok(Self {
            client,
            username,
            password,
        })
    }
    pub async fn login(&self) -> anyhow::Result<()> {
        let sign = self.login_step1().await?;
        let location = if !sign.starts_with("http") {
            self.login_step2(sign.as_str()).await?
        } else {
            sign
        };
        let status = self.login_step3(location.as_str()).await?;
        if status == StatusCode::OK {
            return Ok(());
        }
        // 403 错误
        Err(anyhow!("登录米家账号失败"))
    }

    pub async fn get_devices(&self, country: &str) -> anyhow::Result<()> {
        let api_url = self._get_api_url(country,"/home/device_list");
        let nonce=Utils::gen_nonce();

        let resp = self.client.post(api_url.as_str())
            .send().await?;
        return Ok(());
    }
    fn _get_api_url(&self, country: &str, path: &str) -> String {
        let country_lower = country.trim().to_lowercase();
        let api_url = if country_lower == "cn" {
            "".to_string()
        } else {
            format!("{}.", country_lower)
        };
        format!("https://{}api.io.mi.com/app/{}", api_url, path)
    }


    async fn login_step3(&self, location: &str) -> anyhow::Result<StatusCode> {
        let resp = self.client.get(location).send().await?;
        Ok(resp.status())
    }

    /// 获取sign or location
    async fn login_step1(&self) -> anyhow::Result<String> {
        let url = "https://account.xiaomi.com/pass/serviceLogin?sid=xiaomiio&_json=true";
        let resp = self.client.get(url).send().await.unwrap();
        let text = resp.text().await?;
        println!("{}", text);
        let text = text.as_str().replace(JSON_START, "");
        //转成json
        let mut json_map: serde_json::map::Map<String, Value> = serde_json::from_str(text.as_str()).unwrap();
        if let Some(location) = json_map.remove("location") {
            if let Some(val) = location.as_str() {
                return Ok(val.to_string());
            }
        };
        let sign = json_map.remove("_sign");
        match sign {
            None => {}
            Some(s) => {
                return Ok(s.as_str().unwrap().to_string());
            }
        }
        Err(anyhow::anyhow!("获取sign 失败"))
    }

    async fn login_step2(&self, sign: &str) -> anyhow::Result<String> {
        let url = "https://account.xiaomi.com/pass/serviceLoginAuth2";
        let hash: String = md5::compute(self.password.as_bytes()).encode_hex_upper();


        let mut params = HashMap::new();
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
        let text = resp.text().await?;
        if !text.starts_with(JSON_START) {
            return Err(anyhow::anyhow!("登录失败 请检查用户名密码"));
        };
        let text = text.replace(JSON_START, "");
        //转成json
        let mut json_map: serde_json::map::Map<String, Value> = serde_json::from_str(text.as_str())
            .map_err(|e| anyhow!("json解析失败"))?;

        if let Some(location) = json_map.remove("location") {
            if let Some(l) = location.as_str() {
                return Ok(l.to_string());
            }
        }
        Err(anyhow::anyhow!("获取location 失败"))
    }
}

