use chrono::NaiveDateTime;
use sea_orm::ActiveValue::Set;
use sea_orm::NotSet;
use serde::{Deserialize, Serialize};
use crate::db::entity::hap_bridge::{BonjourStatusFlagWrapper, BridgeCategory, BridgeInfo, PairingsWrapper};
use crate::db::entity::prelude::{HapBridgeActiveModel, HapBridgeModel};

#[derive(Debug, Serialize, Deserialize)]
pub struct HapBridgeTemplate {
    pub bridge_id: Option<i64>,
    pub pin_code: Option<i64>,
    pub port: Option<i64>,
    pub category: Option<BridgeCategory>,
    pub name: Option<String>,
    pub mac: Option<String>,
    pub setup_id: Option<String>,
    pub host: Option<String>,
    pub disabled: Option<bool>,
    ///是否是单配件桥接器
    pub single_accessory: Option<bool>,
    ///配对列表
    pub pairings: Option<PairingsWrapper>,
    pub device_ed25519_keypair: Option<String>,
    pub configuration_number: i64,
    pub status_flag: BonjourStatusFlagWrapper,
    pub max_peers: Option<i64>,
    pub info: BridgeInfo,
    pub create_at: chrono::NaiveDateTime,
    pub update_at: chrono::NaiveDateTime,
}


impl TryFrom<HapBridgeModel> for HapBridgeTemplate {
    type Error = anyhow::Error;

    fn try_from(value: HapBridgeModel) -> Result<Self, Self::Error> {
        Ok(
            Self {
                bridge_id: Some(value.bridge_id),
                pin_code: Some(value.pin_code),
                port: Some(value.port),
                category: Some(value.category),
                name: Some(value.name),
                mac: Some(value.mac),
                setup_id: Some(value.setup_id),
                host: value.host,
                disabled: Some(value.disabled),
                single_accessory: Some(value.single_accessory),
                pairings: Some(value.pairings),
                device_ed25519_keypair: Some(value.device_ed25519_keypair),
                configuration_number: value.configuration_number,
                status_flag: value.status_flag,
                max_peers: value.max_peers,
                info: value.info,
                create_at: value.create_at,
                update_at: value.update_at,
            }
        )
    }
}


impl HapBridgeTemplate {
    pub fn try_into_update_model(self) -> anyhow::Result<HapBridgeActiveModel> {
        Ok(HapBridgeActiveModel{
            bridge_id: Set(self.bridge_id.ok_or(anyhow::anyhow!("bridge_id is required"))?),
            pin_code: self.pin_code.map_or(NotSet, |x| Set(x)),
            port: self.port.map_or(NotSet, |x| Set(x)),
            category: self.category.map_or(NotSet, |x| Set(x)),
            name: self.name.map_or(NotSet, |x| Set(x)),
            mac: self.mac.map_or(NotSet, |x| Set(x)),
            setup_id: self.setup_id.map_or(NotSet, |x| Set(x)),
            host: self.host.map_or(NotSet, |x| Set(Some(x.clone()))),
            disabled: self.disabled.map_or(NotSet, |x| Set(x)),
            single_accessory: self.single_accessory.map_or(NotSet, |x| Set(x)),
            pairings: self.pairings.map_or(NotSet, |x| Set(x)),
            device_ed25519_keypair: self.device_ed25519_keypair.map_or(NotSet, |x| Set(x)),
            configuration_number: Set(self.configuration_number),
            status_flag: Set(self.status_flag),
            max_peers: self.max_peers.map_or(NotSet, |x| Set(Some(x))),
            info: Set(self.info),
            create_at: NotSet,
            update_at: Set(chrono::Local::now().naive_local()),
        })
    }
    pub fn try_into_insert_model(self) -> anyhow::Result<HapBridgeActiveModel> {
        Ok(
            HapBridgeModel {
                bridge_id: self.bridge_id.ok_or(anyhow::anyhow!("bridge_id is required"))?,
                pin_code: self.pin_code.ok_or(anyhow::anyhow!("pin_code is required"))?,
                port: self.port.ok_or(anyhow::anyhow!("port is required"))?,
                category: self.category.ok_or(anyhow::anyhow!("category is required"))?,
                name: self.name.ok_or(anyhow::anyhow!("name is required"))?,
                mac: self.mac.ok_or(anyhow::anyhow!("mac is required"))?,
                setup_id: self.setup_id.ok_or(anyhow::anyhow!("setup_id is required"))?,
                host: self.host,
                disabled: self.disabled.ok_or(anyhow::anyhow!("disabled is required"))?,
                single_accessory: self.single_accessory.ok_or(anyhow::anyhow!("single_accessory is required"))?,
                pairings: self.pairings.ok_or(anyhow::anyhow!("pairings is required"))?,
                device_ed25519_keypair: self.device_ed25519_keypair.ok_or(anyhow::anyhow!("device_ed25519_keypair is required"))?,
                configuration_number: self.configuration_number,
                status_flag: self.status_flag,
                max_peers: self.max_peers,
                info: self.info,
                create_at:chrono::Local::now().naive_local(),
                update_at: chrono::Local::now().naive_local(),
            }.into()
        )
    }
}