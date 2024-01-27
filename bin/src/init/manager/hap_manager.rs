use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use anyhow::{anyhow, Error};
use futures_util::lock::Mutex;
use impl_new::New;
use log::{error, info};
use serde_json::Value;
use tap::TapFallible;
use hap::Config;
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

#[derive(New)]
pub struct ChannelInfo {
    ch_id: i64,
    ruing: bool,
}

pub struct AccessoryInfo {
    /// 配件id
    pub aid: u64,
    /// 配件与js 通信的通道
    pub ch_id: Option<ChannelInfo>,
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
#[derive(Default)]
pub struct HapManageInner {
    server_map: dashmap::DashMap<i64, HapTask>,
    // 配件id关系
    aid_map: dashmap::DashMap<u64, AccessoryInfo>,
    /// 每个配件同时只能运行一个
    aid_ch_map: dashmap::DashMap<u64, ChannelInfo>,
    /// 配件与设备的关系
    dev_aid_map: dashmap::DashMap<i64, u64>,

}

impl HapManageInner {
    pub fn get_bridge_server_config(&self, bid: i64) -> Option<Arc<Mutex<Config>>> {
        self.server_map.get(&bid)
            .map(|i| i.server.config_pointer().clone())
    }
    pub(crate) async fn update_char_value(&self, aid: u64, service_tag: String, char_tag: String, value: Value) -> anyhow::Result<()> {
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
    pub async fn put_accessory_ch(&self, aid: u64, ch_id: i64, status: bool) {
        self.aid_ch_map.insert(aid, ChannelInfo::new(ch_id, status));
    }

    pub async fn get_accessory_ch_id(&self, aid: u64) -> anyhow::Result<i64> {
        self.aid_ch_map
            .get(&aid)
            .map(|i| i.ch_id)
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
    pub async fn stop_server(&self, bid: i64) -> anyhow::Result<()> {
        if let Some((_, task)) = self.server_map.remove(&bid) {
            task.sender
                .send(true)
                .map_err(|e| anyhow!("发送退出指令失败:{:?}", e))?;
        }
        Ok(())
    }
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
                ..Default::default()
            })
        }
    }
}