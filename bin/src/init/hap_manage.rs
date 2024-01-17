use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use anyhow::anyhow;
use log::{error, info};
use serde_json::Value;
use tap::TapFallible;
use hap::server::{IpServer, Server};
use crate::init::hap_init::AccessoryRelation;
use crate::init::HapAccessoryPointer;

pub struct HapTask {
    sender: tokio::sync::oneshot::Sender<bool>,
    server: IpServer,
}

pub struct ServiceTagMap {
    id: u64,
    // tag->id
    char_tags: HashMap<String, u64>,
}

pub struct AccessoryInfo {
    /// 配件id
    pub aid: u64,
    /// 配件与js 通信的通道
    pub ch_id: Option<i64>,
    /// 设备id
    pub device_id: i64,
    /// 桥接器的id
    pub bid: i64,

    ///tags-> service_id,char_ids
    pub service_tag_map: HashMap<String, ServiceTagMap>,

    /// 配件指针
    pub accessory: HapAccessoryPointer,
}


/// hap 设备管理器
/// 移除device 必须移除其对应的hap 设备
pub struct HapManageInner {
    server_map: dashmap::DashMap<i64, HapTask>,
    // 配件id关系
    aid_map: dashmap::DashMap<u64, AccessoryInfo>,
    // aid_bid_map: dashmap::DashMap<u64, i64>,
    aid_ch_id_map: dashmap::DashMap<u64, i64>,
    /// 配件与设备的关系
    dev_aid_map: dashmap::DashMap<i64, u64>,

}

impl HapManageInner {
    pub(crate) async fn update_char_value(&self, aid: u64, service_tag: String, char_tag: String, value: Value) -> anyhow::Result<()> {
        info!("aid map size :{}", self.aid_map.len());

        let accessory = self.aid_map.get(&aid)
            .ok_or(anyhow!("设备:{}不存在",aid))
            .tap_err(|e| error!("{}",e))?;

        // let services = &accessory.service_tag_map;
        // let mut char_ids = HashMap::new();
        let mut lock = accessory.accessory.lock().await;

        let services = lock.get_mut_services_by_tag(service_tag.as_str());
        for svc in services {
            let chs = svc.get_mut_characteristics_by_tag(char_tag.as_str());
            for ch in chs {
                ch.set_value(value.clone()).await?;
            }
        }
        Ok(())
    }
    pub async fn put_accessory_ch(&self, aid: u64, ch_id: i64) {
        self.aid_ch_id_map.insert(aid, ch_id);
    }

    pub async fn get_accessory_ch_id(&self, aid: u64) -> anyhow::Result<i64> {
        self.aid_ch_id_map
            .get(&aid)
            .map(|i| i.clone())
            .ok_or(anyhow!("未运行js脚本"))
    }

    pub async fn refresh_accessory_config(&self, aid: u64) {
        /* if let Some(bid) = self.aid_bid_map.get(&aid) {
             if let Some(server) = self.server_map.get(bid.value()) {
                 server.server.configuration_number_incr().await;
             }
         }*/
    }
    pub async fn close(&self) {}
    pub fn push_server(&self, bid: i64, server: IpServer, accessories: Vec<AccessoryRelation>) {
        let (sender, recv) = tokio::sync::oneshot::channel();
        let server_c = server.clone();
        self.server_map.insert(bid, HapTask {
            server,
            sender,
        });
        for rel in accessories {
            let info = AccessoryInfo {
                aid: rel.aid,
                ch_id: None,
                device_id: rel.device_id,
                bid,
                service_tag_map: Default::default(),
                accessory: rel.accessory,
            };
            self.dev_aid_map.insert(rel.device_id, rel.aid);
            self.aid_map.insert(rel.aid, info);
        }
        info!("aid map size :{}", self.aid_map.len());

        tokio::spawn(async move {
            let task = async move {
                let res = server_c.run_handle().await;
                error!("hap server退出:{:?},res:{:?}",bid, res);
            };
            loop {
                tokio::select! {
                    Ok(val)= recv=>{
                        error!("收到hap服务:{}退出指令,{}",bid,val);
                        break
                    }
                    _= task=>break,
                }
            }
        });
    }
}


#[derive(Clone)]
pub struct HapManage {
    inner: Arc<HapManageInner>,
}



impl Deref for HapManage {
    type Target = HapManageInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl HapManage {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(HapManageInner {
                server_map: Default::default(),
                aid_map: Default::default(),
                aid_ch_id_map: Default::default(),
                dev_aid_map: Default::default(),
            })
        }
    }
}