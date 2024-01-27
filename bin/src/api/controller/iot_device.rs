use axum::extract::{Path, Query, State};
use axum::Json;
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use crate::api::output::{ApiResp, ApiResult};
use crate::api::state::AppState;
use crate::db::entity::prelude::{IotDeviceEntity, IotDeviceActiveModel, IotDeviceColumn, IotDeviceModel};
use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;
use crate::api::params::{AddServiceParam, DisableParam, QueryIotDeviceParam};
use crate::api::results::IotDeviceResult;

pub async fn list(state: State<AppState>, Query(param): Query<QueryIotDeviceParam>) -> ApiResult<Vec<IotDeviceResult>> {
    let mut query = IotDeviceEntity::find();
    if let Some(f) = param.device_type {
        query.query().and_where(IotDeviceColumn::DeviceType.eq(f));
    };
    let list = query
        .all(state.conn())
        .await?;

    let dev_manager = state.device_manager.clone();
    let list: Vec<IotDeviceResult> = list.into_iter().map(|i| {
        let running = dev_manager.is_running(i.device_id);
        IotDeviceResult {
            model: i,
            running,
        }
    }).collect();
    Ok(ApiResp::with_data(list))
}

pub async fn disable(state: State<AppState>, Path(id): Path<i64>, Query(param): Query<DisableParam>) -> ApiResult<()> {
    let model = IotDeviceActiveModel {
        device_id: Set(id),
        disabled: Set(param.disabled),
        ..Default::default()
    };
    IotDeviceEntity::update(model)
        .filter(IotDeviceColumn::DeviceId.eq(id))
        .exec(state.conn()).await?;
    Ok(ApiResp::with_data(()))
}


/// 导出模板

pub async fn export_template(state: State<AppState>, Path(id): Path<i64>) {}