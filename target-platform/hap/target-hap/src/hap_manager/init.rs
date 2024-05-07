use std::sync::Arc;
use anyhow::anyhow;
use futures_util::lock::Mutex;
use log::{error, info, warn};
use serde_json::Value;
use tap::TapFallible;
use hap::{Config, HapType, pointer};
use hap::server::{IpServer, Server};
use crate::hap_manager::{AccessoryInfo, AccessoryRelation, HapManageInner, HapTask};
use crate::HapAccessoryPointer;
use crate::types::HapCharInfo;

impl HapManageInner {
    /// 移除配件
    pub async fn remove_accessory(&self, aid: u64) -> anyhow::Result<()> {
        let info = self.accessory_map.remove(&aid);
        if let Some((_, info)) = info {
            let bid = info.bid;
            if let Some(server) = self.server_map.get_mut(&bid) {
                server.server.remove_accessory(aid).await?;
            };
            info!("移除配件:{},bid:{}", aid, bid);
        };
        Ok(())
    }

    /// 获取连接
    pub fn get_bridge_server_peer(&self, bid: i64) -> Option<pointer::Peers> {
        self.server_map.get(&bid)
            .map(|i| i.server.peers_pointer().clone())
    }
    pub fn get_bridge_server_config(&self, bid: i64) -> Option<Arc<Mutex<Config>>> {
        self.server_map.get(&bid)
            .map(|i| i.server.config_pointer().clone())
    }

    pub(crate) fn update_char_value_by_accessory(&self, accessory: HapAccessoryPointer, sid: u64, cid: u64, value: Value) {
        tokio::spawn(async move {
            match accessory.write()
                .await
                .get_mut_service_by_id(sid)
                .and_then(|s| s.get_mut_characteristic_by_id(cid)) {
                None => {
                    warn!("特征:{}不存在",cid);
                }
                Some(cts) => {
                    //类型转换器,设置值
                    if let Err(e) = cts.set_value(value).await {
                        warn!("设置特征值失败:{:?}",e);
                    }
                }
            };
        });
    }

    ///根据id更新, todo 可重入修复
    pub async fn update_char_value_by_id(&self, aid: u64, sid: u64, ctag: HapType, value: Value) -> anyhow::Result<()> {
        let accessory = self.accessory_map.get(&aid)
            .ok_or(anyhow!("设备:{}不存在",aid))?
            .accessory.clone();
        tokio::spawn(async move {
            match accessory.write()
                .await
                .get_mut_service_by_id(sid) {
                None => {
                    warn!("服务:{}不存在",sid);
                }
                Some(s) => {
                    match s.get_mut_characteristic(ctag) {
                        None => {
                            warn!("特征:{:?}不存在",ctag);
                        }
                        Some(c) => {
                            match c.set_value(value).await {
                                Ok(_) => {}
                                Err(e) => {
                                    warn!("设置特征值失败:{:?}",e);
                                }
                            };
                        }
                    }
                }
            }
        });
        Ok(())
    }

    pub(crate) async fn update_char_value(&self, aid: u64, service_tag: String, char_tag: HapType, value: Value) -> anyhow::Result<()> {
        let accessory = self.accessory_map.get(&aid)
            .ok_or(anyhow!("设备:{}不存在",aid))
            .tap_err(|e| error!("{}",e))?
            .accessory.clone();

        tokio::spawn(async move {
            // let services = &accessory.service_tag_map;
            // let mut char_ids = HashMap::new();
            let mut lock = accessory.write().await;

            let services = lock.get_mut_services_by_tag(service_tag.as_str());
            for svc in services {
                let ch = svc.get_mut_characteristic(char_tag);
                if let Some(ch) = ch {
                    if let Err(e) = ch.set_value(value.clone()).await {
                        warn!("设置特征值失败:{:?}",e);
                    }
                }
            }
        });
        Ok(())
    }
    // pub async fn put_accessory_ch(&self, aid: u64, ch_id: i64, status: bool) {
    //     self.aid_ch_map.insert(aid, ChannelInfo::new(ch_id, status));
    // }
    //
    // pub async fn get_accessory_ch_id(&self, aid: u64) -> anyhow::Result<i64> {
    //     self.aid_ch_map
    //         .get(&aid)
    //         .map(|i| i.ch_id)
    //         .ok_or(anyhow!("未运行js脚本"))
    // }


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
                device_id: rel.device_id,
                bid,
                accessory: rel.accessory,
            };
            self.aid_dev_map.insert(rel.aid, rel.device_id);
            self.accessory_map.insert(rel.aid, info);
        }
        //启动服务
        tokio::spawn(async move {
            let task = async move {
                let res = server_c.run_handle().await;
                error!("hap_platform server退出:{:?},res:{:?}",bid, res);
            };
            loop {
                tokio::select! {
                    Ok(val)= recv=>{
                        info!("收到hap服务:{}退出指令,{}",bid,val);
                        break
                    }
                    _= task=>break,
                }
            }
        });
    }


    pub fn get_hap_default_info(&self, hap: HapType) -> Option<HapCharInfo> {
        self.default_type_info_map.get(&hap).cloned()
    }
}