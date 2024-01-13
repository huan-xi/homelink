use axum::extract::{Path, Query, State};
use axum::Json;
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use crate::api::output::{ApiResp, ApiResult};
use crate::api::state::AppState;
use crate::db::entity::prelude::{HapServiceColumn, HapServiceEntity, HapServiceModel, IotDevice, IotDeviceActiveModel, IotDeviceColumn, IotDeviceModel};
use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;
use crate::api::params::{AddServiceParam, DisableParam};

pub async fn list(state: State<AppState>) -> ApiResult<Vec<IotDeviceModel>> {
    let list = IotDevice::find()
        .all(state.conn()).await?;
    Ok(ApiResp::with_data(list))
}

pub async fn disable(state: State<AppState>, Path(id): Path<i64>, Query(param): Query<DisableParam>) -> ApiResult<()> {
    let model = IotDeviceActiveModel {
        device_id: Set(id),
        disabled: Set(param.disabled),
        ..Default::default()
    };
    IotDevice::update(model)
        .filter(IotDeviceColumn::DeviceId.eq(id))
        .exec(state.conn()).await?;
    Ok(ApiResp::with_data(()))
}
