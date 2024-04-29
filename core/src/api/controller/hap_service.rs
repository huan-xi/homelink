use std::str::FromStr;
use axum::extract::{Path, Query, State};
use axum::Json;
use sea_orm::PaginatorTrait;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, EntityTrait, TransactionTrait, TryIntoModel};
use crate::api::output::{ApiResp, ApiResult, err_msg};
use crate::api::state::AppState;
use crate::db::entity::prelude::{HapCharacteristicColumn, HapCharacteristicEntity, HapServiceActiveModel, HapServiceColumn, HapServiceEntity, HapServiceModel};
use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;
use target_hap::hap_type_wrapper::HapTypeWrapper;
use crate::api::params::{AddServiceParam, DisableParam};
use crate::db::SNOWFLAKE;
use crate::{api_err, err_msg};
use crate::api::params::power::power_add_param::PowerAddParam;


/// 返回 配件服务列表
pub async fn list(state: State<AppState>, Path(id): Path<i64>) -> ApiResult<Vec<HapServiceModel>> {
    let list = HapServiceEntity::find()
        .filter(HapServiceColumn::AccessoryId.eq(id))
        .all(state.conn()).await?;

    Ok(ApiResp::with_data(list))
}

/// 添加服务 add_service
/// ///修改服务
pub async fn add_service(state: State<AppState>, Json(param): Json<PowerAddParam>) -> ApiResult<()> {
    let mut active_model = param.to_active_model::<HapServiceEntity, HapServiceActiveModel>()?;
    active_model.id = Set(SNOWFLAKE.next_id());
    active_model.disabled = Set(false);
    let svc_type = active_model.service_type.clone().take()
        .ok_or(api_err!("服务类型不能为空"))?;
    HapTypeWrapper::from_str(svc_type.as_str())?;
    let tag = active_model.tag.clone().take().ok_or(api_err!("tag不能为空"))?;
    let accessory_id = active_model.accessory_id.clone().take().ok_or(api_err!("配件id不能为空"))?;
    //检测tag是否重复
    let count = HapServiceEntity::find()
        .filter(HapServiceColumn::Tag.eq(tag.clone())
            .and(HapServiceColumn::AccessoryId.eq(accessory_id)))
        .count(state.conn())
        .await?;
    if count > 0 {
        return Err(api_err!("tag:{:?}已存在", tag));
    }
    active_model.insert(state.conn()).await?;

    Ok(ApiResp::with_data(()))
}

pub async fn delete(state: State<AppState>, Path(id): Path<i64>) -> ApiResult<()> {
    // 删除所有设备
    let txn = state.conn().begin().await?;
    let _ = HapCharacteristicEntity::delete_many()
        .filter(HapCharacteristicColumn::ServiceId.eq(id))
        .exec(&txn)
        .await?;
    let model = HapServiceActiveModel {
        id: Set(id),
        ..Default::default()
    };
    model.delete(&txn).await?;
    txn.commit().await?;
    Ok(ApiResp::with_data(()))
}

pub async fn disable(state: State<AppState>, Path(id): Path<i64>, Query(param): Query<DisableParam>) -> ApiResult<()> {
    let service = HapServiceEntity::find_by_id(id).one(state.conn())
        .await?
        .ok_or(api_err!("服务不存在"))?;
    let model = HapServiceActiveModel {
        id: Set(id),
        disabled: Set(param.disabled),
        ..Default::default()
    };
    model.update(state.conn()).await?;
    state.hap_manager.refresh_accessory_config(service.accessory_id as u64).await;
    Ok(ApiResp::with_data(()))
}