use axum::extract::{Path, Query, State};
use axum::Json;
use log::info;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, EntityTrait, TransactionTrait};
use crate::api::output::{ApiResp, ApiResult, err_msg};
use crate::api::state::AppState;
use crate::db::entity::prelude::{HapCharacteristicColumn, HapCharacteristicEntity, HapServiceActiveModel, HapServiceColumn, HapServiceEntity, HapServiceModel};
use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;
use crate::api::params::{AddServiceParam, DisableParam};
use crate::db::SNOWFLAKE;
use crate::{api_err, err_msg};


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
    let type_str = format!("{:?}", param.service_type);
    info!("param:{:?}", param);
    let meta = state.hap_metadata
        .services
        .get(type_str.as_str())
        .ok_or(api_err!("未找到服务类型:{}",type_str))?;
    let mut names = vec![];
    for x in param.characteristics.iter() {
        names.push(x.name
            .clone()
            .ok_or(api_err!("特征:{:?}名称不能为空", x.characteristic_type))?
        );
    }
    let mut list = vec![];
    meta.characteristics.required_characteristics
        .iter()
        .for_each(|c| {
            if !names.iter().any(|c1| c1.as_str() == c.as_str()) {
                list.push(c.to_string());
            }
        });
    if list.len() > 0 {
        return err_msg!("必填特征 [{}] 未设置", list.join(","));
    }
    let service_id = SNOWFLAKE.next_id();
    let mut chs = vec![];
    for i in param.characteristics.into_iter() {
        let mut model = i.into_model(service_id)?;
        model.cid = Set(SNOWFLAKE.next_id());
        chs.push(model);
    }
    let model = HapServiceActiveModel {
        id: Set(service_id),
        accessory_id: Set(param.accessory_id),
        name: Set(param.name),
        tag: Default::default(),
        memo: Set(param.memo),
        service_type: Set(param.service_type),
        disabled: Set(false),
    };
    let txn = state.conn().begin().await?;
    HapServiceEntity::insert(model).exec(&txn).await?;
    HapCharacteristicEntity::insert_many(chs).exec(&txn).await?;
    txn.commit().await?;

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