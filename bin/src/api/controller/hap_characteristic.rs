use axum::extract::{Path, Query, State};
use axum::Json;
use log::info;
use sea_orm::ColumnTrait;
use sea_orm::ActiveValue::Set;
use crate::api::output::{ApiResp, ApiResult, err_msg, err_msg_string};
use crate::api::state::AppState;
use crate::db::entity::prelude::{HapAccessoryActiveModel, HapAccessoryEntity, HapCharacteristicActiveModel, HapCharacteristicColumn, HapCharacteristicEntity, HapCharacteristicModel, HapServiceColumn, HapServiceEntity, HapServiceModel};
use sea_orm::{ActiveModelTrait, EntityTrait, PaginatorTrait, QueryFilter};
use crate::api::errors::ApiError;
use crate::api::params::{AddCharacteristicParam, AddServiceParam, DisableParam};
use crate::db::entity::hap_characteristic::{BleToSensorParam, DbBleValueType, MappingMethod, MappingParam, MiotSpecParam, Property};
use crate::db::SNOWFLAKE;


pub async fn update(state: State<AppState>, Json(param): Json<AddCharacteristicParam>) -> ApiResult<()> {
   /* let service_id=
    let mapping_param = match &param.mapping_method {
        MappingMethod::Unknown => {
            panic!("mapping_method 不能为 Unknown")
        }
        MappingMethod::MIotSpec => {
            match param.mapping_property.clone() {
                None => {
                    return Err(ApiError::StrMsg("mapping_property 不能为空"));
                }
                Some(s) => {
                    MappingParam::MIotSpec(MiotSpecParam {
                        property: s,
                    })
                }
            }
        }
        MappingMethod::BleToSensor => {
            MappingParam::BleToSensor(BleToSensorParam {
                ble_value_type: match param.ble_value_type.clone() {
                    None => {
                        return Err(ApiError::StrMsg("ble_value_type 不能为空"));
                    }
                    Some(s) => {
                        s
                    }
                },
            })
        }
    };


    let model = HapCharacteristicActiveModel {
        cid: Set(param.cid.unwrap()),
        service_id: Set(param.service_id),
        disabled: Set(false),
        name: Set(param.name),
        characteristic_type: Set(param.characteristic_type),
        mapping_method: Set(param.mapping_method),
        format: Set(param.format),
        fixed_value: Default::default(),
        unit: Default::default(),
        min_value: Set(param.min_value),
        max_value: Set(param.max_value),
        max_len: Default::default(),
        unit_convertor: Default::default(),
        convertor_param: Default::default(),
        mapping_param: Set(Some(mapping_param)),
    };
    HapCharacteristicEntity::update(model).exec(state.conn()).await?;
    return Ok(ApiResp::with_data(()));*/
    todo!();
}

pub async fn add(state: State<AppState>, Path(id): Path<i64>, Json(param): Json<AddCharacteristicParam>) -> ApiResult<()> {
    let service_id = id;
    let count = HapCharacteristicEntity::find()
        .filter(HapCharacteristicColumn::ServiceId.eq(service_id)
            .and(HapCharacteristicColumn::CharacteristicType.eq(param.characteristic_type)))
        .count(state.conn()).await?;
    if count > 0 {
        return err_msg_string(format!("服务已存在特征类型:{:?}", param.characteristic_type));
    }

    // SwitchService::new();
    info!("param:{:?}", param);
    let mut model = param.into_model(service_id)?;
    model.cid = Set(SNOWFLAKE.next_id());
    HapCharacteristicEntity::insert(model).exec(state.conn()).await?;
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