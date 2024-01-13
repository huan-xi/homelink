use axum::extract::{Path, Query, State};
use axum::Json;
use sea_orm::{ActiveModelTrait, EntityTrait, ModelTrait};
use sea_orm::ActiveValue::Set;
use tap::Conv;
use crate::api::output::{ApiResp, ApiResult};
use crate::api::params::{AddHapAccessoryParam, DisableParam};
use crate::api::results::HapAccessoryResult;
use crate::api::state::AppState;
use crate::db::entity::prelude::{HapAccessoryActiveModel, HapAccessoryEntity, HapAccessoryModel, HapAccessoryRelation, HapBridgeEntity, IotDevice, IotDeviceActiveModel, IotDeviceColumn, IotDeviceModel};
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
        .all(state.conn()).await?;
    let mut result = vec![];
    for (model, bridge) in list.into_iter() {
        let device = model.find_related(IotDevice).one(state.conn()).await?;
        result.push(HapAccessoryResult {
            model,
            bridge,
            device,
        });
    }
    Ok(ApiResp::with_data(result))
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