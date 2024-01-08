use axum::extract::{Path, Query, State};
use axum::Json;
use log::info;
use sea_orm::ActiveValue::Set;
use crate::api::output::{ApiResp, ApiResult};
use crate::api::state::AppState;
use crate::db::entity::prelude::{HapAccessoryActiveModel, HapAccessoryEntity, HapCharacteristicActiveModel, HapCharacteristicColumn, HapCharacteristicEntity, HapCharacteristicModel, HapServiceColumn, HapServiceEntity, HapServiceModel};
use sea_orm::{ActiveModelTrait, EntityTrait, QueryFilter};
use crate::api::errors::ApiError;
use crate::api::params::{AddCharacteristicParam, AddServiceParam, DisableParam};
use crate::db::entity::hap_characteristic::{BleToSensorParam, DbBleValueType, MappingMethod, MappingParam, MiotSpecParam, Property};
use crate::db::SNOWFLAKE;
use sea_orm::ColumnTrait;

pub async fn add(state: State<AppState>, Json(param): Json<AddCharacteristicParam>) -> ApiResult<()> {
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


    // SwitchService::new();
    info!("param:{:?}", param);
    let model = HapCharacteristicActiveModel {
        cid: Set(SNOWFLAKE.next_id()),
        service_id: Set(param.service_id),
        disabled: Set(false),
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

    HapCharacteristicEntity::insert(model).exec(state.conn()).await?;
    return Ok(ApiResp::with_data(()));
}

pub async fn list(state: State<AppState>, axum::extract::Path(id): axum::extract::Path<i64>) -> ApiResult<Vec<HapCharacteristicModel>> {
    let list = HapCharacteristicEntity::find()
        .filter(HapCharacteristicColumn::ServiceId.eq(id))
        .all(state.conn()).await?;
    Ok(ApiResp::with_data(list))
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