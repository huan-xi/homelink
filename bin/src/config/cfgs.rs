use serde::Deserialize;
use std::env::VarError;
use std::fmt::format;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::pin::pin;
use hap::Pin;

// const CFG_FILE: &str = "config.toml";
const CFG_FILE: &str = "config.toml";

/// server 配置文件
#[derive(Debug, Deserialize)]
pub struct Server {
    /// 服务器名称
    pub name: String,
    pub version: String,
    /// 服务器(IP地址:端口)
    /// `0.0.0.0:5514`
    pub address: String,
    /// 服务器ssl
    pub ssl: bool,
    /// 响应数据gzip
    // pub content_gzip: bool,
    /// api 前缀  例如："/api"
    pub api_prefix: String,
    /// dab 前缀
    // pub dav_prefix: String,
    /// 数据存储目录 /data
    pub data_dir: String,
    pub db_schema: Option<String>,
}

/// 数据库
#[derive(Debug, Deserialize)]
pub struct Database {
    /// 数据库连接
    pub link: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct HapConfig {
    /// 数据库连接
    pub pin: Option<String>,
    pub name: Option<String>,
}

impl HapConfig {
    pub fn name(&self) -> String {
        match self.name.as_ref() {
            None => {
                "HG Bridge".to_string()
            }
            Some(name) => {
                name.to_string()
            }
        }
    }
    /// 获取pin码
    pub fn pin(&self) -> Pin {
        let arr = match self.pin.as_ref() {
            Some(pin) => {
                let mut arr: [u8; 8] = [0; 8]; // 初始化一个长度为8的u8数组
                for (i, c) in pin.chars().enumerate() {
                    if i < 8 {
                        arr[i] = c.to_digit(10).unwrap() as u8
                    }
                }
                arr
            }
            None => {
                [1, 1, 1, 2, 2, 3, 3, 3]
            }
        };
        Pin::new(arr).unwrap()
    }
}

/// 配置文件
#[derive(Debug, Deserialize)]
pub struct Configs {
    /// 程序配置
    pub server: Server,
    pub hap_config: HapConfig,
    // pub database: Database,
}

impl Configs {
    pub fn init() -> Self {
        let path = match std::env::var("CFG_FILE") {
            Ok(s) => s,
            Err(_) => CFG_FILE.to_string(),
        };
        let tmp = fs::canonicalize(path.as_str()).expect(format!("配置文件:{}路径错误", path.as_str()).as_str());
        let p = tmp.as_path();

        let mut file = match File::open(p) {
            Ok(f) => f,
            Err(e) => panic!("不存在配置文件：{:?}，错误信息：{}", p, e),
        };
        let mut cfg_contents = String::new();
        match file.read_to_string(&mut cfg_contents) {
            Ok(s) => s,
            Err(e) => panic!("读取配置文件失败，错误信息：{}", e),
        };
        toml::from_str(&cfg_contents).expect("解析配置文件错误")
    }
}
