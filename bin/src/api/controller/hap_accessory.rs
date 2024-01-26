use anyhow::anyhow;
use axum::extract::{Path, Query, State};
use axum::Json;
use log::info;
use sea_orm::{ActiveModelTrait, EntityTrait, ModelTrait, QueryOrder};
use sea_orm::ActiveValue::Set;
use crate::api::output::{ApiResp, ApiResult};
use crate::api::params::{AddHapAccessoryParam, DisableParam, UpdateHapAccessoryParam};
use crate::api::results::HapAccessoryResult;
use crate::api::state::AppState;
use crate::db::entity::prelude::{HapAccessoryActiveModel, HapAccessoryColumn, HapAccessoryEntity, HapAccessoryModel, HapAccessoryRelation, HapBridgeEntity, IotDeviceEntity};
use crate::db::SNOWFLAKE;


pub async fn add(state: State<AppState>, Json(param): Json<AddHapAccessoryParam>) -> ApiResult<()> {
    let mut model = param.into_model()?;
    model.aid = Set(SNOWFLAKE.next_id());
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

pub async fn update_script() {

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