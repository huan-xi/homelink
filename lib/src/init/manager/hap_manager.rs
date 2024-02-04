use std::collections::HashMap;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;
use anyhow::{anyhow, Error};
use futures_util::lock::Mutex;
use impl_new::New;
use log::{error, info, warn};
use serde_json::Value;
use tap::TapFallible;
use hap::{Config, HapType, pointer};
use hap::characteristic::{Format, HapCharacteristic, Perm, Unit};
use hap::server::{IpServer, Server};
use hap::service::HapService;
use hap_metadata::hap_metadata;
use hap_metadata::metadata::HapMetadata;
use crate::db::entity::hap_characteristic::HapCharInfo;
use crate::hap::iot::iot_characteristic::CharacteristicValue;
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
    accessory_map: dashmap::DashMap<u64, AccessoryInfo>,
    /// 每个配件同时只能运行一个
    aid_ch_map: dashmap::DashMap<u64, ChannelInfo>,
    /// 配件与设备的关系
    dev_aid_map: dashmap::DashMap<i64, u64>,

    default_type_info_map: HashMap<HapType, HapCharInfo>,
}

impl HapManageInner {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}


impl HapManageInner {
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
    pub(crate) async fn update_char_value_by_id(&self, aid: u64, sid: u64, ctag: HapType, value: Value) -> anyhow::Result<()> {
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
                                Ok(_) => {
                                    info!("设置特征值成功");
                                }
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

    pub(crate) async fn update_char_value(&self, aid: u64, service_tag: String, char_tag: String, value: Value) -> anyhow::Result<()> {
        let accessory = self.accessory_map.get(&aid)
            .ok_or(anyhow!("设备:{}不存在",aid))
            .tap_err(|e| error!("{}",e))?;

        // let services = &accessory.service_tag_map;
        // let mut char_ids = HashMap::new();
        let mut lock = accessory.accessory.write().await;

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
            self.accessory_map.insert(rel.aid, info);
        }
        tokio::spawn(async move {
            let task = async move {
                let res = server_c.run_handle().await;
                error!("hap server退出:{:?},res:{:?}",bid, res);
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

fn get_default_type_info_map(meta: Arc<HapMetadata>) -> anyhow::Result<HashMap<HapType, HapCharInfo>> {
    let mut map = HashMap::new();
    let meta = hap_metadata().unwrap();

    let chars = meta.characteristics;
    for (name, char) in chars {
        let str = char.short_uuid.as_str().trim_start_matches('0');
        match HapType::from_str(str) {
            Ok(hap_type) => {
                let format_str = format!("\"{}\"", char.format.as_str());
                let format: Format = serde_json::from_str(format_str.as_str())?;
                let unit = char
                    .units
                    .as_ref()
                    .map(|i| {
                        let unit = format!("\"{}\"", i.as_str());
                        let unit: Result<Unit, serde_json::Error> = serde_json::from_str(unit.as_str());
                        unit
                    })
                    .transpose()?;

                let min_value = char.min_value.as_ref().map(|i| {
                    CharacteristicValue::try_format(format, i.clone())
                        .map(|i| i.value)
                }).transpose()?;
                let max_value = char.max_value.as_ref().map(|i| {
                    CharacteristicValue::try_format(format, i.clone())
                        .map(|i| i.value)
                }).transpose()?;
                let step_value = char.step_value.as_ref().map(|i| {
                    CharacteristicValue::try_format(format, i.clone())
                        .map(|i| i.value)
                }).transpose()?;
                let max_len = char.max_length
                    .as_ref()
                    .and_then(|i| i.as_u64().map(|i| i as u16));
                let perms = hap_metadata::get_perms(char.properties as u64);
                let perm_str = serde_json::to_string(&perms)?;
                let perms: Vec<Perm> = serde_json::from_str(perm_str.as_str())?;
                let in_values = meta.characteristic_in_values.get(name.as_str());
                let out_values = meta.characteristic_out_values.get(name.as_str());
                let mut valid_values = None;
                if let Some(values) = in_values {
                    //枚举
                    valid_values = Some(values.clone().into_values().collect());
                };

                if let Some(values) = out_values {
                    //枚举
                    valid_values = Some(values.clone().into_values().collect());
                };


                let info = HapCharInfo {
                    format,
                    unit,
                    min_value,
                    max_value,
                    step_value,
                    max_len,
                    max_data_len: None,
                    valid_values,
                    valid_values_range: None,
                    ttl: None,
                    perms,
                    pid: None,
                };

                map.insert(hap_type, info);
            }
            Err(e) => {
                // let name = pascal_case(x.name.as_str());
                // println!("error,name:{name}:{:?}", x);
            }
        }
    }
    return Ok(map);
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
    pub fn new(arc: Arc<HapMetadata>) -> Self {
        let default_type_info_map = get_default_type_info_map(arc)
            .expect("create default get error");
        Self {
            inner: Arc::new(HapManageInner {
                default_type_info_map,
                ..Default::default()
            })
        }
    }
}


#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::str::FromStr;
    use hap::{HapType};
    use hap::characteristic::{Format, Perm, Unit};
    use hap_metadata::hap_metadata;
    use crate::db::entity::hap_characteristic::HapCharInfo;
    use crate::hap::iot::iot_characteristic::CharacteristicValue;

    #[test]
    pub fn test_default() -> anyhow::Result<()> {
        // SecuritySystemCurrentStateCharacteristic::new();

        let mut map = HashMap::new();
        let meta = hap_metadata().unwrap();

        let chars = meta.characteristics;
        for (name, char) in chars {
            let str = char.short_uuid.as_str().trim_start_matches('0');
            match HapType::from_str(str) {
                Ok(hap_type) => {
                    let format_str = format!("\"{}\"", char.format.as_str());
                    let format: Format = serde_json::from_str(format_str.as_str())?;
                    let unit = char
                        .units
                        .as_ref()
                        .map(|i| {
                            let unit = format!("\"{}\"", i.as_str());
                            let unit: Result<Unit, serde_json::Error> = serde_json::from_str(unit.as_str());
                            unit
                        })
                        .transpose()?;

                    let min_value = char.min_value.as_ref().map(|i| {
                        CharacteristicValue::try_format(format, i.clone())
                            .map(|i| i.value)
                    }).transpose()?;
                    let max_value = char.max_value.as_ref().map(|i| {
                        CharacteristicValue::try_format(format, i.clone())
                            .map(|i| i.value)
                    }).transpose()?;
                    let step_value = char.step_value.as_ref().map(|i| {
                        CharacteristicValue::try_format(format, i.clone())
                            .map(|i| i.value)
                    }).transpose()?;
                    let max_len = char.max_length
                        .as_ref()
                        .and_then(|i| i.as_u64().map(|i| i as u16));
                    let perms = hap_metadata::get_perms(char.properties as u64);
                    let perm_str = serde_json::to_string(&perms)?;
                    let perms: Vec<Perm> = serde_json::from_str(perm_str.as_str())?;
                    let in_values = meta.characteristic_in_values.get(name.as_str());
                    let out_values = meta.characteristic_out_values.get(name.as_str());
                    let mut valid_values = None;
                    if let Some(values) = in_values {
                        //枚举
                        valid_values = Some(values.clone().into_values().collect());
                    };

                    if let Some(values) = out_values {
                        //枚举
                        valid_values = Some(values.clone().into_values().collect());
                    };


                    let info = HapCharInfo {
                        format,
                        unit,
                        min_value,
                        max_value,
                        step_value,
                        max_len,
                        max_data_len: None,
                        valid_values,
                        valid_values_range: None,
                        ttl: None,
                        perms,
                        pid: None,
                    };

                    map.insert(hap_type, info);
                }
                Err(e) => {
                    // let name = pascal_case(x.name.as_str());
                    // println!("error,name:{name}:{:?}", x);
                }
            }
        }

        let a = map.get(&HapType::SecuritySystemCurrentState);
        println!("{:?}", a);
        Ok(())
    }
}