use std::collections::HashMap;
use anyhow::anyhow;
use axum::extract::{Path, Query, State};
use axum::Json;
use log::info;
use sea_orm::*;
use crate::api::errors::{ApiError, ApiErrorInner};
use crate::api::output::{ApiResult, ok_data};
use crate::api::params::GetTemplateParam;
use crate::api::params::template::CheckTemplateParam;
use crate::api::results::{CheckTemplateResult, TemplateResult};
use crate::api::state::AppState;
use crate::api_err;
use crate::db::entity::prelude::{IotDeviceColumn, IotDeviceEntity};
use crate::template::hl_template::{HlDeviceTemplate, TemplateFormat};

pub async fn check_template_add(state: State<AppState>, Json(param): Json<TemplateResult>) -> ApiResult<CheckTemplateResult> {
    let template = param.text.as_str();
    let template: HlDeviceTemplate = param.format.parse(template)
        .map_err(|e| api_err!("模板格式错误:{:?}", e))?;
    let mut new_devices = vec![];
    let mut new_accessories = vec![];
    for device in template.devices {
        new_devices.push(device.clone());
        for accessory in device.accessories {
            new_accessories.push(accessory.clone());
        }
    }
    ok_data(CheckTemplateResult {
        new_devices,
        new_accessories,
    })
}

pub async fn check_template_update(state: State<AppState>, Json(param): Json<CheckTemplateParam>) -> ApiResult<()> {
    let template = param.text.as_str();
    let template: HlDeviceTemplate = param.format.parse(template)
        .map_err(|e| api_err!("模板格式错误:{:?}", e))?;
    let temp_id = template.id.clone();
    //batch_id => {
    // device:
    // }
    let mut batch_map = HashMap::new();

    for device in template.devices {
        //先检查是否存在设备
        let old_device_list = IotDeviceEntity::find()
            .filter(
                IotDeviceColumn::TempId.eq(temp_id.to_string())
                    .and(IotDeviceColumn::SourcePlatform.eq(param.source_platform.clone()))
                    .and(IotDeviceColumn::SourceId.eq(param.source_id.clone()))
                    .and(IotDeviceColumn::Tag.eq(device.tag.clone()))
            )
            .all(state.conn())
            .await?;
        info!("old_list:{:?}", old_device_list);
        for dev_model in old_device_list {
            batch_map.insert(dev_model.temp_batch_id.clone(), dev_model);
        }

        for accessory in device.accessories {
            //是否存在该配件,


            /*let old = HapAccessoryEntity::find()
                .filter(HapAccessoryColumn::DeviceId.eq(model.device_id.clone().unwrap())
                    .and(HapAccessoryColumn::Tag.eq(accessory.tag.clone().unwrap()))
                    .and(HapAccessoryColumn::TempId.eq(accessory.temp_id.clone().unwrap())))
                .one(txn)
                .await?;*/
        }
    }

    // 读取新增配件,对新增配件需要添加 bridgeId

    // template.devices

    ok_data(())
}


pub async fn get_text(state: State<AppState>, Path(id): Path<String>, Query(param): Query<GetTemplateParam>) -> ApiResult<TemplateResult> {
    let temp = match state.template_manager
        .templates.get(&id) {
        None => {
            return Err(ApiError::ApiErrorInner(ApiErrorInner::new(1, "模板不存在".to_string())));
        }
        Some(s) => { s.clone() }
    };
    let text = match param.format {
        TemplateFormat::Yaml => {
            serde_yaml::to_string(&temp)
                .map_err(|e| anyhow!("yaml 转换失败"))?
        }
        TemplateFormat::Toml => {
            toml::to_string(&temp)
                .map_err(|e| anyhow!("yaml 转换失败"))?
        }
    };
    ok_data(TemplateResult {
        text,
        format: param.format,
    })
}

// 获取模板格式
pub async fn get(state: State<AppState>, Path(id): Path<String>) -> ApiResult<HlDeviceTemplate> {
    let temp = state.template_manager
        .templates.get(&id)
        .ok_or(api_err!("模板不存在"))?
        .clone();
    // let list = MiAccountEntity::find().all(state.conn()).await?;
    ok_data(temp)
}