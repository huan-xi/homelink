mod init;
mod method;
mod default_char_info;

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::ops::Deref;
use std::sync::Arc;
use log::error;
use tap::TapFallible;
use tokio::sync::Mutex;
use hap::{HapType, MdnsResponder, pointer, RawMdnsResponder};
use hap::pointer::MdnsResponderPtr;
use hap::server::IpServer;
use hap_metadata::hap_metadata;
use hap_metadata::metadata::HapMetadata;
use hl_integration::hl_device::manager::ISourceDeviceManager;
use crate::hap_manager::default_char_info::get_default_type_info_map;
use crate::HapAccessoryPointer;
use crate::types::HapCharInfo;

pub struct HapTask {
    sender: tokio::sync::oneshot::Sender<bool>,
    pub server: IpServer,
}

pub struct AccessoryRelation {
    pub aid: u64,
    pub device_id: i64,
    pub accessory: HapAccessoryPointer,
}

pub struct AccessoryInfo {
    /// 配件id
    pub aid: u64,
    /// 设备id
    pub device_id: i64,
    /// 桥接器的id
    pub bid: i64,
    ///tags-> service_id,char_ids
    // pub service_tag_map: HashMap<String, ServiceTagMap>,

    /// 配件指针
    pub accessory: HapAccessoryPointer,
}


/// hap_platform 设备管理器
/// 移除device 必须移除其对应的hap 设备

pub struct HapManageInner {
    pub hap_mata: Arc<HapMetadata>,
    pub server_map: dashmap::DashMap<i64, HapTask>,
    // 配件id关系
    pub accessory_map: dashmap::DashMap<u64, AccessoryInfo>,
    // 每个配件同时只能运行一个
    // aid_ch_map: dashmap::DashMap<u64, ChannelInfo>,
    /// 配件与设备的关系
    aid_dev_map: dashmap::DashMap<u64, i64>,
    default_type_info_map: HashMap<HapType, HapCharInfo>,

    mdns_responder: Mutex<Option<Arc<Mutex<RawMdnsResponder>>>>,
}


#[derive(Clone)]
pub struct HapManage {
    inner: Arc<HapManageInner>,
}

impl HapManage {
    pub async fn get_mdns(&self) -> anyhow::Result<Arc<Mutex<RawMdnsResponder>>> {
        let mut prt = self.mdns_responder.lock().await;
        if prt.is_some() {
            return Ok(prt.as_ref().unwrap().clone());
        }
        let raw = match prt.as_ref() {
            Some(s) => {
                s.clone()
            }
            None => {
                let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 3, 180));
                let raw = RawMdnsResponder::new_with_ip_list(vec![ip])
                    .tap_err(|e| error!("mdns 启动错误:{e:?}"))?;
                let arc = Arc::new(Mutex::new(raw));
                *prt = Some(arc.clone());
                arc
            }
        };
        Ok(raw)
    }
}

impl HapManage {
    pub fn new() -> Self {
        let meta = Arc::new(hap_metadata().unwrap());
        let default_type_info_map = get_default_type_info_map(meta.clone())
            .expect("create default get error");
        Self {
            inner: Arc::new(HapManageInner {
                hap_mata: meta,
                server_map: Default::default(),
                accessory_map: Default::default(),
                aid_dev_map: Default::default(),
                default_type_info_map,
                mdns_responder: Default::default(),
            })
        }
    }


    pub async fn init(&self) -> anyhow::Result<()> {
        Ok(())
    }
}


impl Deref for HapManage {
    type Target = HapManageInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}