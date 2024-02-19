use anyhow::anyhow;
use axum::extract::{Path, Query, State};
use axum::Json;
use log::info;
use sea_orm::{ActiveModelTrait, EntityTrait, ModelTrait, QueryOrder, TransactionTrait};
use sea_orm::ActiveValue::Set;
use crate::api::output::{ApiResp, ApiResult, ok_data};
use crate::api::params::{AddHapAccessoryParam, DisableParam, UpdateHapAccessoryParam};
use crate::api::results::HapAccessoryResult;
use crate::api::state::AppState;
use crate::db::entity::prelude::{HapAccessoryActiveModel, HapAccessoryColumn, HapAccessoryEntity, HapAccessoryModel, HapAccessoryRelation, HapBridgeEntity, HapCharacteristicColumn, HapCharacteristicEntity, HapServiceActiveModel, HapServiceColumn, HapServiceEntity, IotDeviceEntity};
use crate::db::SNOWFLAKE;
use sea_orm::*;

pub async fn add(state: State<AppState>, Json(param): Json<AddHapAccessoryParam>) -> ApiResult<()> {
    let mut model = param.into_model()?;
    model.aid = Set(SNOWFLAKE.next_id());
    model.create_at = Set(chrono::Local::now().naive_local());
    model.insert(state.conn()).await?;
    Ok(ApiResp::with_data(()))
}


pub async fn list(state: State<AppState>) -> ApiResult<Vec<HapAccessoryResult>> {
    // HapBridge
    let list = HapAccessoryEntity::find()
        .find_also_related(HapBridgeEntity)
        .order_by_desc(HapAccessoryColumn::Aid)
        .all(state.conn()).await?;
    let mut result = vec![];
    for (model, bridge) in list.into_iter() {
        let device = model.find_related(IotDeviceEntity).one(state.conn()).await?;
        result.push(HapAccessoryResult {
            model,
            bridge,
            device,
        });
    }
    Ok(ApiResp::with_data(result))
}

pub async fn detail(state: State<AppState>, Path(id): Path<i64>) -> ApiResult<HapAccessoryModel> {
    let model = HapAccessoryEntity::find_by_id(id)
        .one(state.conn()).await?
        .ok_or(anyhow!("该配件不存在"))?;
    Ok(ApiResp::with_data(model))
}

pub async fn update(state: State<AppState>, Path(id): Path<i64>, Json(param): Json<UpdateHapAccessoryParam>) -> ApiResult<()> {
    info!("param:{:?}",id);
    let mut model = param.into_model(id)?;
    model.update(state.conn()).await?;
    Ok(ApiResp::with_data(()))
}

pub async fn update_script() {}

///删除配件
pub async fn delete(state: State<AppState>, Path(id): Path<i64>) -> ApiResult<()> {
    // 删除所有设备
    let txn = state.conn().begin().await?;
    let svc = HapServiceEntity::find()
        .filter(HapServiceColumn::AccessoryId.eq(id))
        .all(&txn)
        .await?;
    let svc_ids = svc.iter().map(|i| i.id).collect::<Vec<i64>>();

    let _ = HapCharacteristicEntity::delete_many()
        .filter(HapCharacteristicColumn::ServiceId.is_in(svc_ids.clone()))
        .exec(&txn)
        .await?;
    let _ = HapServiceEntity::delete_many()
        .filter(HapServiceColumn::Id.is_in(svc_ids))
        .exec(&txn)
        .await?;
    //删除配件
    let model = HapAccessoryActiveModel {
        aid: Set(id),
        ..Default::default()
    };
    model.delete(&txn).await?;
    txn.commit().await?;
    ok_data(())
}

pub async fn disable(state: State<AppState>, Path(id): Path<i64>, Query(param): Query<DisableParam>) -> ApiResult<()> {
    let model = HapAccessoryActiveModel {
        aid: Set(id),
        disabled: Set(param.disabled),
        ..Default::default()
    };
    model.update(state.conn()).await?;
    Ok(ApiResp::with_data(()))
}