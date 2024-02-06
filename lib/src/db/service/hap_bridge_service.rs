use anyhow::anyhow;
use sea_orm::ActiveValue::Set;
use sea_orm::*;
use sea_orm::prelude::Expr;
use hap::BonjourStatusFlag;
use crate::db::entity::hap_bridge::{BonjourStatusFlagWrapper, BridgeCategory, BridgeInfo, PairingsWrapper};
use crate::db::entity::prelude::{HapBridgeActiveModel, HapBridgeColumn, HapBridgeEntity, HapBridgeModel};
use crate::{api_err, err_msg};
use crate::db::init::SeaQuery;
use crate::db::SNOWFLAKE;
use crate::hap::rand_utils::{rand_mac_addr, rand_pin_code, rand_setup_id};

pub async fn create_hap_bridge<C>(conn: &C, pin_code: Option<String>,
                               category: BridgeCategory,
                               name: String, single_accessory: bool) -> anyhow::Result<HapBridgeModel>
    where C: ConnectionTrait {
    let count = HapBridgeEntity::find()
        .filter(HapBridgeColumn::Name.eq(&name))
        .count(conn)
        .await?;
    if count > 0 {
        return Err(anyhow!("该名称桥接器已存在"));
    }

    let pin_code = match pin_code.filter(|x| !x.is_empty()) {
        None => {
            rand_pin_code() as i64
        }
        Some(s) => {
            if s.len() != 8 {
                return Err(anyhow!("pin_code length must be 8"));
            }
            s.parse::<i64>().map_err(|e| anyhow!("pin code 格式错误"))?
        }
    };
    let mac = rand_mac_addr();
    let builder = conn.get_database_backend();
    let st = SeaQuery::select().from(HapBridgeEntity)
        .expr(Expr::col(HapBridgeColumn::Port).max())
        .to_owned();
    let stmt = builder.build(&st);
    let result = conn.query_one(stmt).await?;
    let default = 30000;

    let port = match result {
        None => default,
        Some(r) => {
            match r.try_get_by_index::<Option<i64>>(0)? {
                None => default,
                Some(s) => s + 1
            }
        }
    };
    let bid = SNOWFLAKE.next_id();
    let bytes = hap::Config::default().device_ed25519_keypair.to_bytes();
    let device_ed25519_keypair = hex::encode(bytes);
    let model = HapBridgeActiveModel {
        bridge_id: Set(bid),
        pin_code: Set(pin_code),
        port: Set(port),
        category: Set(category),
        name: Set(name),
        mac: Set(mac.to_string()),
        setup_id: Set(rand_setup_id()),
        device_ed25519_keypair: Set(device_ed25519_keypair),
        single_accessory: Set(single_accessory),
        pairings: Set(PairingsWrapper::default()),
        host: Set(None),
        max_peers: Set(None),
        info: Set(BridgeInfo {
            serial_number: bid.to_string(),
            model: "homelink".to_string(),
            manufacturer: "homelink".to_string(),
            software_revision: Some("v1.0".to_string()),
        }),
        disabled: Set(false),
        configuration_number: Set(1),
        status_flag: Set(BonjourStatusFlagWrapper(BonjourStatusFlag::NotPaired)),
        ..Default::default()
    };

    // let model = hap_bridge.clone().into_active_model();
    HapBridgeEntity::insert(model.clone()).exec(conn).await?;
    let hap_bridge = model.try_into_model()?;
    Ok(hap_bridge)
}