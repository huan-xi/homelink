use anyhow::anyhow;
use sea_orm::{ActiveEnum, EntityTrait};
use hap::BonjourStatusFlag;
use target_hap::hap_manager::HapManage;
use crate::api::results::HapBridgeResult;
use crate::api::state::AppState;
use sea_orm::*;
use crate::db::entity::prelude::{HapAccessoryColumn, HapAccessoryEntity, HapBridgeColumn, HapBridgeEntity, HapBridgeModel};
use crate::hap::rand_utils::gen_homekit_setup_uri_default;


pub async fn add_check(state: &AppState, name: &str, pin_code: i64) -> anyhow::Result<()> {
    let count = HapBridgeEntity::find()
        .filter(HapBridgeColumn::Name.eq(name))
        .count(state.conn())
        .await?;
    if count > 0 {
        return Err(anyhow!("该名称桥接器已存在"));
    }
    let count = HapBridgeEntity::find()
        .filter(HapBridgeColumn::PinCode.eq(pin_code))
        .count(state.conn())
        .await?;
    if count > 0 {
        return Err(anyhow!("该pinCode桥接器已存在"));
    }
    Ok(())
}

pub async fn to_model_result(state: &AppState, manager: &HapManage, model: HapBridgeModel) -> anyhow::Result<HapBridgeResult> {
    let config = manager.get_bridge_server_config(model.bridge_id);
    let peers = manager.get_bridge_server_peer(model.bridge_id);
    let peers = match peers {
        None => vec![],
        Some(s) => {
            s.read().await.iter().map(|i| i.clone()).collect()
        }
    };
    let setup_uri = gen_homekit_setup_uri_default(
        model.pin_code as u64, model.category.to_value() as u64,
        model.setup_id.clone());
    let running = config.is_some();
    let is_paired = model.status_flag.0 != BonjourStatusFlag::NotPaired;
    let accessory_count = HapAccessoryEntity::find()
        .filter(HapAccessoryColumn::BridgeId.eq(model.bridge_id))
        .count(state.conn())
        .await?;
    Ok(HapBridgeResult {
        model,
        setup_uri,
        peers,
        running,
        accessory_count,
        is_paired,
    })
}