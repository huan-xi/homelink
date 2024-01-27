use axum::extract::{Path, Query, State};
use axum::Json;
use deno_runtime::deno_core::Op;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveEnum, ConnectionTrait, EntityTrait, IntoActiveModel, PaginatorTrait, QueryResult, TransactionTrait};
use sea_orm::prelude::Expr;
use crate::api::output::{ApiResp, ApiResult, ok_data, output_err_msg};
use crate::api::params::{AddHapBridgeParam, DisableParam};
use crate::api::state::AppState;
use crate::db::entity::prelude::{HapBridgeActiveModel, HapBridgeColumn, HapBridgeEntity, HapBridgeModel, IotDeviceEntity};
use crate::{api_err, err_msg};
use crate::api::results::HapBridgeResult;
use crate::db::SNOWFLAKE;
use crate::hap::rand_utils::{gen_homekit_setup_uri_default, rand_mac_addr, rand_pin_code, rand_setup_id};
use crate::db::init::SeaQuery;
use crate::init::hap_init::{add_hap_bridge};

/// 添加桥接器
pub async fn add(state: State<AppState>, Json(param): Json<AddHapBridgeParam>) -> ApiResult<()> {
    let pin_code = match param.pin_code.filter(|x| !x.is_empty()) {
        None => {
            rand_pin_code() as i64
        }
        Some(s) => {
            if s.len() != 8 {
                return err_msg!("pin_code length must be 8");
            }
            s.parse::<i64>().map_err(|e| api_err!("pin code 格式错误"))?
        }
    };
    let mac = rand_mac_addr();
    let conn = state.conn();
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
    let hap_bridge = HapBridgeModel {
        bridge_id: bid,
        pin_code,
        port,
        category: param.category,
        name: param.name,
        mac: mac.to_string(),
        setup_id: rand_setup_id(),
        disabled: false,
    };

    let model = hap_bridge.clone().into_active_model();
    HapBridgeEntity::insert(model).exec(state.conn()).await?;
    add_hap_bridge(state.conn(), hap_bridge, state.hap_manager.clone(), state.device_manager.clone())
        .await
        .map_err(|e| api_err!("添加成功,启动失败{e}")).unwrap();
    ok_data(())
}

/// 重启桥接器
pub async fn restart(state: State<AppState>, Path(id): Path<i64>) -> ApiResult<()> {
    state.hap_manager.stop_server(id).await?;
    let model = HapBridgeEntity::find_by_id(id).one(state.conn()).await?;
    let hap_bridge = model.ok_or(api_err!("桥接器不存在"))?;
    add_hap_bridge(state.conn(), hap_bridge, state.hap_manager.clone(), state.device_manager.clone())
        .await
        .map_err(|e| api_err!("停止成功,启动失败{e}")).unwrap();
    ok_data(())
}

pub async fn list(state: State<AppState>) -> ApiResult<Vec<HapBridgeResult>> {
    let list = HapBridgeEntity::find()
        .all(state.conn())
        .await?;
    let manager = state.hap_manager.clone();
    let list = list.into_iter().map(|i| {
        let config = manager.get_bridge_server_config(i.bridge_id);

        let setup_uri = gen_homekit_setup_uri_default(
            i.pin_code as u64, i.category.to_value() as u64,
            i.setup_id.clone());
        HapBridgeResult {
            model: i,
            setup_uri,
            running: config.is_some(),
        }
    }).collect();

    Ok(ApiResp::with_data(list))
}


pub async fn disable(state: State<AppState>, Path(id): Path<i64>, Query(param): Query<DisableParam>) -> ApiResult<()> {
    let model = HapBridgeEntity::find_by_id(id).one(state.conn()).await?;
    let mut hap_bridge = model.ok_or(api_err!("桥接器不存在"))?;
    hap_bridge.disabled = param.disabled;

    let update_model = HapBridgeActiveModel {
        bridge_id: Set(id),
        disabled: Set(param.disabled),
        ..Default::default()
    };

    HapBridgeEntity::update(update_model).exec(state.conn()).await?;
    if param.disabled {
        state.hap_manager.stop_server(id).await?;
    } else {
        add_hap_bridge(state.conn(), hap_bridge, state.hap_manager.clone(), state.device_manager.clone())
            .await
            .map_err(|e| api_err!("停止成功,启动失败{e}")).unwrap();
    }
    ok_data(())
}