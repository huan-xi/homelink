use axum::extract::{Path, Query, State};
use axum::Json;
use sea_orm::EntityTrait;
use crate::api::output::{ApiResp, ApiResult};
use crate::api::params::{ DisableParam};
use crate::api::state::AppState;
use crate::db::entity::prelude::{HapBridgeEntity, HapBridgeModel, IotDevice, IotDeviceActiveModel, IotDeviceModel};
use crate::hap::iot_hap_accessory::IotDeviceAccessory;

/*pub async fn add(state: State<AppState>, Json(param): Json<AddIotDeviceParam>) -> ApiResult<()> {

}*/
pub async fn list(state: State<AppState>) -> ApiResult<Vec<HapBridgeModel>> {
    let list = HapBridgeEntity::find()
        .all(state.conn()).await?;
    Ok(ApiResp::with_data(list))
}

pub async fn disable(state: State<AppState>, Path(id): Path<i64>, Query(param): Query<DisableParam>) -> ApiResult<()> {
    todo!();
}