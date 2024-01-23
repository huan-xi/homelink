use std::sync::{Arc, Mutex};
use anyhow::anyhow;
use axum::extract::{Query, State};
use dashmap::mapref::entry::Entry;
use deno_runtime::deno_core::error::AnyError;
use log::warn;
use sea_orm::ActiveValue::Set;
use sea_orm::{EntityTrait, JsonValue};
use serde::Serialize;
use serde_json::Value;
use miot_spec::cloud::MiCloud;
use crate::api::output::{ApiResult, ok_data};
use crate::api::params::SyncDeviceParam;
use crate::api::state::AppState;
use crate::config::context::{get_app_context, get_data_dir};
use crate::db::entity::prelude::{MiotDeviceActiveModel, MiotDeviceEntity};

#[derive(serde::Deserialize, Debug, Serialize)]
pub struct MiotDeviceResult {
    pub did: String,
    pub token: String,
    pub name: String,
    pub model: String,
    pub localip: Option<String>,
    pub mac: Option<String>,
    pub is_online: Option<bool>,
    pub full: Option<String>,
}

pub async fn new_cloud(username: String, password: Option<String>) -> anyhow::Result<MiCloud> {
    let path = format!("{}/mi_cloud", get_data_dir());
    MiCloud::new(path.as_str(), username, password).await
}

pub async fn sync_mi_devices1(state: State<AppState>, Query(param): Query<SyncDeviceParam>) -> ApiResult<i32> {
    ok_data(1)
}

/// 读取米家设备
pub async fn sync_mi_devices(state: State<AppState>, Query(param): Query<SyncDeviceParam>) -> ApiResult<i32> {
    let state_c = state.clone();
    let account = param.account;

    let cloud = match state.mi_cloud_map.entry(account.clone()) {
        Entry::Occupied(e) => {
            e.into_ref()
        }
        Entry::Vacant(e) => {
            let cloud = new_cloud(account.clone(), None).await?;
            e.insert(Arc::new(cloud))
        }
    }.clone();


    let resp = get_device(state_c, account).await?;

    let mut count = 0;
    let devices = resp.as_object()
        .and_then(|obj| obj.get("result"))
        .and_then(|res| res.as_object())
        .and_then(|res| res.get("list"))
        .and_then(|list| list.as_array())
        .ok_or(anyhow::anyhow!("米家云返回数据格式错误"))?;
    for device in devices {
        //处理设备 MiotDeviceResult
        let dev = serde_json::from_value::<MiotDeviceResult>(device.clone());
        match dev {
            Ok(dev_result) => {
                count = count + 1;
                let a = MiotDeviceEntity::find_by_id(dev_result.did.clone()).one(state.conn()).await?;
                let text = serde_json::to_string(&dev_result)
                    .map_err(|e| anyhow!("parse error"))?;
                let module = MiotDeviceActiveModel {
                    did: Set(dev_result.did),
                    token: Set(dev_result.token),
                    name: Set(dev_result.name),
                    model: Set(dev_result.model),
                    localip: Set(dev_result.localip),
                    mac: Set(dev_result.mac),
                    is_online: Set(dev_result.is_online.unwrap_or(false)),
                    full: Set(text),
                };
                if a.is_none() {
                    MiotDeviceEntity::insert(module).exec(state.conn()).await?;
                } else {
                    MiotDeviceEntity::update(module).exec(state.conn()).await?;
                }
            }
            Err(e) => {
                warn!("设备数据解析错误:{:?}", e);
            }
        }
    }
    ok_data(count)
}

async fn get_device(state: State<AppState>, account: String) -> Result<Value, AnyError> {
    let cloud = match state.mi_cloud_map.entry(account.clone()) {
        Entry::Occupied(e) => {
            e.into_ref()
        }
        Entry::Vacant(e) => {
            let cloud = new_cloud(account.clone(), None).await?;
            e.insert(Arc::new(cloud))
        }
    }.clone();
    // 日志buffer->
    cloud.get_devices().await
}
/*
/// 一键转换
pub fn convert_to_iot_device() {}


/// 更新转换
pub fn update_iot_device() {}

/// 登入账号
pub async fn login_mi_account() {}

pub fn delete_device() {}*/