mod init;
mod method;
mod default_char_info;

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use hap::HapType;
use hap::server::IpServer;
use hap_metadata::hap_metadata;
use hap_metadata::metadata::HapMetadata;
use hl_integration::hl_device::manager::ISourceDeviceManager;
// use hl_integration::hl_device::manager::IDeviceManager;
use crate::hap_manager::default_char_info::get_default_type_info_map;
use crate::HapAccessoryPointer;
use crate::types::HapCharInfo;

pub struct HapTask {
    sender: tokio::sync::oneshot::Sender<bool>,
    server: IpServer,
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

pub struct HapManageInner{
    hap_mata: Arc<HapMetadata>,
    server_map: dashmap::DashMap<i64, HapTask>,
    // 配件id关系
    accessory_map: dashmap::DashMap<u64, AccessoryInfo>,
    // 每个配件同时只能运行一个
    // aid_ch_map: dashmap::DashMap<u64, ChannelInfo>,
    /// 配件与设备的关系
    dev_aid_map: dashmap::DashMap<i64, u64>,
    default_type_info_map: HashMap<HapType, HapCharInfo>,
}


#[derive(Clone)]
pub struct HapManage{
    inner: Arc<HapManageInner>,
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
                dev_aid_map: Default::default(),
                default_type_info_map,
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