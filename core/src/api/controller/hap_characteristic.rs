use axum::extract::{Path, Query, State};
use axum::Json;
use log::info;
use sea_orm::ColumnTrait;
use sea_orm::ActiveValue::Set;
use crate::api::output::{ApiResp, ApiResult, err_msg, err_msg_string};
use crate::api::state::AppState;
use crate::db::entity::prelude::{HapAccessoryActiveModel, HapAccessoryEntity, HapCharacteristicActiveModel, HapCharacteristicColumn, HapCharacteristicEntity, HapCharacteristicModel, HapServiceColumn, HapServiceEntity, HapServiceModel};
use sea_orm::{ActiveModelTrait, EntityTrait, PaginatorTrait, QueryFilter};
use crate::api::params::{CharacteristicParam, AddServiceParam, DisableParam};
use crate::api::params::power::power_add_param::PowerAddParam;
use crate::api::params::power::power_update_param::PowerUpdateParam;
use crate::api_err;
use crate::db::SNOWFLAKE;


pub async fn update(state: State<AppState>, Json(param): Json<PowerUpdateParam>) -> ApiResult<()> {
    let mut model = param.to_active_model::<HapCharacteristicEntity, HapCharacteristicActiveModel>()?;
    model.not_set(HapCharacteristicColumn::ServiceId);
    model.update(state.conn()).await?;
    Ok(ApiResp::with_data(()))
}

pub async fn add(state: State<AppState>, Json(param): Json<PowerAddParam>) -> ApiResult<()> {
    let mut active_model = param.to_active_model::<HapCharacteristicEntity, HapCharacteristicActiveModel>()?;

    let service_id = active_model.service_id.clone().take().ok_or(api_err!("服务id不能为空"))?;
    let characteristic_type = active_model.characteristic_type.clone().take().ok_or(api_err!("特征类型不能为空"))?;

    let count = HapCharacteristicEntity::find()
        .filter(HapCharacteristicColumn::ServiceId.eq(service_id)
            .and(HapCharacteristicColumn::CharacteristicType.eq(characteristic_type.clone())))
        .count(state.conn())
        .await?;
    if count > 0 {
        return err_msg_string(format!("服务已存在特征类型:{:?}", characteristic_type.as_str()));
    }

    // SwitchService::new();
    info!("param:{:?}", param);
    active_model.cid = Set(SNOWFLAKE.next_id());
    active_model.disabled = Set(false);
    HapCharacteristicEntity::insert(active_model).exec(state.conn()).await?;
    return Ok(ApiResp::with_data(()));
}

pub async fn list(state: State<AppState>, axum::extract::Path(id): axum::extract::Path<i64>) -> ApiResult<Vec<HapCharacteristicModel>> {
    let list = HapCharacteristicEntity::find()
        .filter(HapCharacteristicColumn::ServiceId.eq(id))
        .all(state.conn()).await?;
    Ok(ApiResp::with_data(list))
}

/// 删除配件
pub async fn delete(state: State<AppState>, Path(id): Path<i64>) -> ApiResult<()> {
    let model = HapCharacteristicActiveModel {
        cid: Set(id),
        ..Default::default()
    };
    model.delete(state.conn()).await?;
    Ok(ApiResp::with_data(()))
}

pub async fn disable(state: State<AppState>, Path(id): Path<i64>, Query(param): Query<DisableParam>) -> ApiResult<()> {
    let model = HapCharacteristicActiveModel {
        cid: Set(id),
        disabled: Set(param.disabled),
        ..Default::default()
    };
    model.update(state.conn()).await?;
    Ok(ApiResp::with_data(()))
}