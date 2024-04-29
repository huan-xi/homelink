use std::str::FromStr;
use anyhow::anyhow;
use axum::extract::{Path, Query, State};
use axum::Json;
use log::info;
use sea_orm::{ActiveModelTrait, EntityTrait, ModelTrait, QueryOrder, TransactionTrait};
use sea_orm::ActiveValue::Set;
use crate::api::output::{ApiResp, ApiResult, ok_data};
use crate::api::params::{DisableParam, GetTemplateParam};
use crate::api::results::{HapAccessoryResult, TemplateResult};
use crate::api::state::AppState;
use crate::db::entity::prelude::{HapAccessoryActiveModel, HapAccessoryColumn, HapAccessoryEntity, HapAccessoryModel, HapAccessoryRelation, HapBridgeEntity, HapCharacteristicColumn, HapCharacteristicEntity, HapServiceActiveModel, HapServiceColumn, HapServiceEntity, IotDeviceEntity};
use crate::db::SNOWFLAKE;
use sea_orm::*;
use hap::server::Server;
use crate::api::errors::ApiError;
use crate::api::params::power::power_add_param::PowerAddParam;
use crate::api::params::power::power_query_param::PowerQueryParam;
use crate::api::params::power::power_update_param::PowerUpdateParam;
use crate::db::entity::hap_accessory::ModelDelegateParamVec;
use crate::template::hap::accessory::AccessoryTemplate;
use crate::template::hap::service::ServiceTemplate;
use crate::template::hl_template::{TemplateFormat};


pub async fn get_template(state: State<AppState>, Path(id): Path<i64>, Query(param): Query<GetTemplateParam>) -> ApiResult<TemplateResult> {
    let accessory = HapAccessoryEntity::find_by_id(id)
        .one(state.conn())
        .await?
        .ok_or(anyhow!("该配件不存在"))?;
    //设置服务
    let services = HapServiceEntity::find()
        .filter(HapServiceColumn::AccessoryId.eq(id))
        .find_with_related(HapCharacteristicEntity)
        .all(state.conn())
        .await?;

    let service_temp = services.into_iter()
        .map(|(svc, chars)| {
            ServiceTemplate::try_from_model(svc, chars)
        }).collect::<Result<Vec<ServiceTemplate>, anyhow::Error>>()?;

    let template = AccessoryTemplate::try_from_model(accessory, service_temp)?;
    let text = param.format.format_to_str(&template)?;
    ok_data(TemplateResult {
        text,
        format: param.format,
    })
}


pub async fn add(state: State<AppState>, Json(param): Json<PowerAddParam>) -> ApiResult<()> {
    let mut model = param.to_active_model::<HapAccessoryEntity, HapAccessoryActiveModel>()?;
    model.aid = Set(SNOWFLAKE.next_id());
    model.disabled = Set(false);
    model.hap_model_delegates = Set(ModelDelegateParamVec(vec![]));
    model.create_at = Set(chrono::Local::now().naive_local());
    model.update_at = Set(chrono::Local::now().naive_local());
    model.insert(state.conn()).await?;
    Ok(ApiResp::with_data(()))
}

pub async fn list(state: State<AppState>, Query(param): Query<PowerQueryParam>) -> ApiResult<Vec<HapAccessoryResult>> {
    let query = param.into_query::<HapAccessoryEntity>()?;
    let list = query
        .find_also_related(HapBridgeEntity)
        .all(state.conn())
        .await?;

    let mut result = vec![];
    for (model, bridge) in list.into_iter() {
        let device = model.find_related(IotDeviceEntity)
            .one(state.conn())
            .await?;
        //读取配件状态
        let mut running = false;
        if let Some(b) = &bridge {
            if let Some(v) = state.hap_manager.server_map.get(&b.bridge_id) {
                let id = model.aid as u64;
                running = v.server.has_accessory(&id).await;
            }
        }

        result.push(HapAccessoryResult {
            model,
            running,
            bridge,
            device,
        });
    }
    Ok(ApiResp::with_data(result))
}

pub async fn detail(state: State<AppState>, Path(id): Path<i64>) -> ApiResult<HapAccessoryModel> {
    let model = HapAccessoryEntity::find_by_id(id)
        .one(state.conn()).await?
        .ok_or(anyhow!("该配件不存在"))?;
    Ok(ApiResp::with_data(model))
}

/// toml 编辑的template
pub async fn update_by_template(state: State<AppState>, Json(param): Json<TemplateResult>) -> ApiResult<()> {
    let template: AccessoryTemplate = match param.format {
        TemplateFormat::Yaml => {
            serde_yaml::from_str(param.text.as_str()).map_err(|e| ApiError::msg(e.to_string()))?
        }
        TemplateFormat::Toml => {
            toml::from_str(param.text.as_str()).map_err(|e| ApiError::msg(e.to_string()))?
        }
    };
    state.template_manager.update_accessory(state.conn(), template).await?;

    ok_data(())
}

pub async fn update(state: State<AppState>,  Json(param): Json<PowerUpdateParam>) -> ApiResult<()> {
    let mut model = param.to_active_model::<HapAccessoryEntity, HapAccessoryActiveModel>()?;
    model.update_at = Set(chrono::Local::now().naive_local());
    model.update(state.conn()).await?;
    Ok(ApiResp::with_data(()))
}

pub async fn update_script() {}

///删除配件
pub async fn delete(state: State<AppState>, Path(id): Path<i64>) -> ApiResult<()> {
    // 删除所有设备
    let txn = state.conn().begin().await?;
    let svc = HapServiceEntity::find()
        .filter(HapServiceColumn::AccessoryId.eq(id))
        .all(&txn)
        .await?;
    let svc_ids = svc.iter().map(|i| i.id).collect::<Vec<i64>>();

    let _ = HapCharacteristicEntity::delete_many()
        .filter(HapCharacteristicColumn::ServiceId.is_in(svc_ids.clone()))
        .exec(&txn)
        .await?;
    let _ = HapServiceEntity::delete_many()
        .filter(HapServiceColumn::Id.is_in(svc_ids))
        .exec(&txn)
        .await?;
    //删除配件
    let model = HapAccessoryActiveModel {
        aid: Set(id),
        ..Default::default()
    };
    model.delete(&txn).await?;
    txn.commit().await?;
    ok_data(())
}

pub async fn disable(state: State<AppState>, Path(id): Path<i64>, Query(param): Query<DisableParam>) -> ApiResult<()> {
    let model = HapAccessoryActiveModel {
        aid: Set(id),
        disabled: Set(param.disabled),
        ..Default::default()
    };
    model.update(state.conn()).await?;
    //移除
    if param.disabled {
        state.hap_manager.remove_accessory(id as u64).await?;
    };

    Ok(ApiResp::with_data(()))
}