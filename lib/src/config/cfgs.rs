use serde::Deserialize;
use std::env::VarError;
use std::fmt::format;
use std::{env, fs};
use std::fs::File;
use std::io::Read;
use std::pin::pin;
use log::info;
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


/// 配置文件
#[derive(Debug, Deserialize)]
pub struct Configs {
    /// 程序配置
    pub server: Server,
    // pub hap_config: HapConfig,
    // pub database: Database,
}

impl Configs {
    pub fn init() -> Self {
        let path = std::env::var("CFG_FILE").unwrap_or_else(|_| CFG_FILE.to_string());
        return match fs::canonicalize(path.as_str()) {
            Ok(path) => {
                let p = path.as_path();
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
            Err(_) => {
                Self::default_config()
            }
        };
        // let tmp = fs::canonicalize(path.as_str()).expect(format!("配置文件:{}路径错误", path.as_str()).as_str());
    }
    pub fn default_config() -> Self {
        let mut config = Configs {
            server: Server {
                name: "homelink".to_string(),
                version: "0.1.0".to_string(),
                address: "0.0.0.0:5514".to_string(),
                ssl: false,
                api_prefix: "/api".to_string(),
                data_dir: "./data".to_string(),
                db_schema: None,
            },
        };
        if let Ok(var) = env::var("DATA_DIR") {
            config.server.data_dir = var;
        }
        info!("默认配置:{:?}",config);
        config
    }
}
