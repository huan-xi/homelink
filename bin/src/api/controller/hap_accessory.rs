use axum::extract::{Path, Query, State};
use sea_orm::{ActiveModelTrait, EntityTrait, ModelTrait};
use sea_orm::ActiveValue::Set;
use tap::Conv;
use crate::api::output::{ApiResp, ApiResult};
use crate::api::params::DisableParam;
use crate::api::results::HapAccessoryResult;
use crate::api::state::AppState;
use crate::db::entity::prelude::{HapAccessoryActiveModel, HapAccessoryEntity, HapAccessoryModel, HapAccessoryRelation, HapBridge, IotDevice, IotDeviceActiveModel, IotDeviceColumn, IotDeviceModel};


pub async fn list(state: State<AppState>) -> ApiResult<Vec<HapAccessoryResult>> {
    // HapBridge
    let list = HapAccessoryEntity::find()
        .find_also_related(HapBridge)
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
        id: Set(id),
        disabled: Set(param.disabled),
        ..Default::default()
    };
    model.update(state.conn()).await?;
    Ok(ApiResp::with_data(()))
}