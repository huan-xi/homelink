use std::ops::Deref;
use std::sync::Arc;
use anyhow::anyhow;
use log::{error, info};
use hap::server::{IpServer, Server};
use crate::init::hap_init::AccessoryRelation;
use crate::js_engine::channel::hap_channel::ToHapModuleSender;

pub struct HapTask {
    sender: tokio::sync::oneshot::Sender<bool>,
    server: IpServer,
}

pub struct AccessoryModule {
    pub sender: Arc<ToHapModuleSender>,
    pub exit_ch: tokio::sync::oneshot::Sender<bool>,
}

/// hap 设备管理器
/// 移除device 必须移除其对应的hap 设备
pub struct HapManageInner {
    server_map: dashmap::DashMap<i64, HapTask>,
    // 配件id关系
    aid_map: dashmap::DashMap<u64, AccessoryRelation>,
    aid_bid_map: dashmap::DashMap<u64, i64>,
    // 配件与设备的关系
    dev_aid_map: dashmap::DashMap<i64, u64>,
    accessory_module: dashmap::DashMap<u64, AccessoryModule>,
}

impl HapManageInner {
    pub async fn get_accessory_module(&self, aid: u64) -> anyhow::Result<Arc<ToHapModuleSender>> {
        self.accessory_module.get(&aid)
            .map(|i| i.sender.clone())
            .ok_or(anyhow!("未运行js脚本"))
    }

    pub async fn refresh_accessory_config(&self, aid: u64) {
        if let Some(bid) = self.aid_bid_map.get(&aid) {
            if let Some(server) = self.server_map.get(bid.value()) {
                server.server.configuration_number_incr().await;
            }
        }
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
            self.dev_aid_map.insert(rel.device_id, rel.aid);
            self.aid_bid_map.insert(rel.aid, bid);
            self.aid_map.insert(rel.aid, rel);
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
                server_map: Default::default(),
                aid_map: Default::default(),
                aid_bid_map: Default::default(),
                dev_aid_map: Default::default(),
                accessory_module: Default::default(),
            })
        }
    }
}