use axum::extract::{Path, Query, State};
use axum::Json;
use log::info;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, EntityTrait};
use crate::api::output::{ApiResp, ApiResult};
use crate::api::state::AppState;
use crate::db::entity::prelude::{HapServiceActiveModel, HapServiceColumn, HapServiceEntity, HapServiceModel};
use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;
use crate::api::params::{AddServiceParam, DisableParam};
use crate::db::SNOWFLAKE;


/// 返回 配件服务列表
pub async fn list(state: State<AppState>, axum::extract::Path(id): axum::extract::Path<i64>) -> ApiResult<Vec<HapServiceModel>> {
    let list = HapServiceEntity::find()
        .filter(HapServiceColumn::AccessoryId.eq(id))
        .all(state.conn()).await?;

    Ok(ApiResp::with_data(list))
}

/// 添加服务 add_service
/// ///修改服务
pub async fn add_service(state: State<AppState>, Json(param): Json<AddServiceParam>) -> ApiResult<()> {
    info!("param:{:?}", param);
    let model = HapServiceActiveModel {
        id: Set(SNOWFLAKE.next_id()),
        accessory_id: Set(param.accessory_id),
        name: Set(param.name),
        hap_type: Set(param.service_type),
        disabled: Set(false),
    };
    HapServiceEntity::insert(model).exec(state.conn()).await?;
    Ok(ApiResp::with_data(()))
}


pub async fn disable(state: State<AppState>, Path(id): Path<i64>, Query(param): Query<DisableParam>) -> ApiResult<()> {
    let model = HapServiceActiveModel {
        id: Set(id),
        disabled: Set(param.disabled),
        ..Default::default()
    };
    model.update(state.conn()).await?;
    Ok(ApiResp::with_data(()))
}