use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

use anyhow::anyhow;
use axum::body::HttpBody;
use axum::extract::{Path, Query, State};
use axum::Json;
use log::{error, warn};
use sea_orm::{ActiveModelTrait, Condition, EntityTrait, JsonValue, PaginatorTrait, QueryFilter};
use sea_orm::ActiveValue::Set;
use sea_orm::prelude::DateTimeUtc;
use tap::TapFallible;
use tokio::net::UdpSocket;
use miot_proto::device::miot_spec_device::MiotDeviceType;
use miot_proto::proto::transport::udp_iot_spec_proto::UdpMiotSpecProtocol;


use crate::api::output::{ApiResult, err_msg, ok_data};
use crate::api::params::{AccountParam, DidParam, MiConvertByTemplateParam, MiConvertToIotParam};
use crate::api::params::power::power_query_param::PowerQueryParam;
use crate::api::params::power::power_update_param::PowerUpdateParam;
use crate::api::results::{MiotDeviceModelResult, MiotDeviceResult, TemplateResult};
use crate::api::state::AppState;
use crate::api_err;
use crate::db::entity::iot_device::{DeviceParam, DeviceType, IotDeviceType, SourcePlatform};
use crate::db::entity::iot_device::DeviceParam::{MiGatewayParam, WifiDeviceParam};
use crate::db::entity::mi_account::MiAccountStatus;
use crate::db::entity::prelude::{IotDeviceActiveModel, IotDeviceEntity, MiAccountActiveModel, MiAccountColumn, MiAccountEntity, MiAccountModel, MiotDeviceActiveModel, MiotDeviceEntity, MiotDeviceModel};
use crate::db::SNOWFLAKE;
use crate::init::manager::template_manager::{ApplyTemplateOptions, SourcePlatformModel};
use crate::template::hl_template::HlDeviceTemplate;
// use crate::template::miot_template::HlDeviceTemplate;
// use crate::init::manager::template_manager::{ApplyTemplateOptions, BridgeMode, SourcePlatformModel};

// accounts
pub async fn update_account(state: State<AppState>, Json(param): Json<PowerUpdateParam>) -> ApiResult<()> {
    let mut model = param.to_active_model::<MiAccountEntity, MiAccountActiveModel>()?;
    model.update_at = Set(DateTimeUtc::from(chrono::Local::now()));
    MiAccountEntity::update(model)
        .exec(state.conn())
        .await?;

    ok_data(())
}

pub async fn delete_account(state: State<AppState>, Query(param): Query<AccountParam>) -> ApiResult<()> {
    let account = MiAccountActiveModel {
        account: Set(param.account),
        ..Default::default()
    };
    account.delete(state.conn()).await?;
    ok_data(())
}

pub async fn change_password(state: State<AppState>, Json(param): Json<AccountParam>) -> ApiResult<()> {
    let account = MiAccountActiveModel {
        account: Set(param.account),
        password: Set(param.password.ok_or(api_err!("密码不能为空"))?),
        ..Default::default()
    };
    MiAccountEntity::update(account)
        .exec(state.conn())
        .await?;
    ok_data(())
}

pub async fn add_account(state: State<AppState>, Json(param): Json<AccountParam>) -> ApiResult<()> {
    let count = MiAccountEntity::find_by_id(param.account.clone())
        .count(state.conn())
        .await?;
    if count > 0 {
        return err_msg("账号已存在");
    };
    let account = MiAccountActiveModel {
        account: Set(param.account),
        password: Set(param.password.ok_or(api_err!("密码不能为空"))?),
        status: Set(MiAccountStatus::NotLogin),
        ..Default::default()
    };
    MiAccountEntity::insert(account)
        .exec(state.conn())
        .await?;

    ok_data(())
}

pub async fn login(state: State<AppState>, Json(param): Json<AccountParam>) -> ApiResult<()> {
    //登入
    state.mi_account_manager.login(param.account.as_str()).await?;
    ok_data(())
}


pub async fn accounts(state: State<AppState>) -> ApiResult<Vec<MiAccountModel>> {
    let list = MiAccountEntity::find().all(state.conn()).await?;
    ok_data(list)
}

pub async fn sync_mi_devices(state: State<AppState>, Query(param): Query<AccountParam>) -> ApiResult<i32> {
    let account = param.account;
    let cloud = state.mi_account_manager.get_cloud(account.as_str()).await?;
    let resp = cloud.read().await.get_devices().await?;

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
                // let text = serde_json::to_string(&dev_result).map_err(|e| anyhow!("parse error"))?;
                let module = MiotDeviceActiveModel {
                    did: Set(dev_result.did),
                    token: Set(dev_result.token),
                    name: Set(dev_result.name),
                    model: Set(dev_result.model),
                    localip: Set(dev_result.localip),
                    mac: Set(dev_result.mac),
                    is_online: Set(dev_result.is_online.unwrap_or(false)),
                    user_id: Set(account.clone()),
                    full: Set(device.clone()),
                    ..Default::default()
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


/// 读取米家设备
pub async fn handshake(state: State<AppState>, Json(param): Json<DidParam>) -> ApiResult<()> {
    let model = MiotDeviceEntity::find_by_id(param.did)
        .one(state.conn())
        .await?
        .ok_or(api_err!("设备不存在"))?;
    let ip = model.localip
        .filter(|ip| !ip.is_empty())
        .ok_or(api_err!("设备无ip"))?;
    let addr: std::net::SocketAddr = format!("{}:{}", ip, 54321).parse()
        .map_err(|f| anyhow!("ip格式错误"))?;
    let mut socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect(addr).await?;
    UdpMiotSpecProtocol::discover(&socket, Duration::from_secs(2))
        .await
        .map_err(|f| api_err!("握手失败"))?;
    ok_data(())
}

pub async fn list(state: State<AppState>, Query(param): Query<PowerQueryParam>) -> ApiResult<Vec<MiotDeviceModelResult>> {
    let condition = param.get_condition::<MiotDeviceEntity>()?;

    let list = MiotDeviceEntity::find()
        .filter(condition)
        .all(state.conn())
        .await?;
    let mut results = vec![];
    for model in list {
        let has_template = state.template_manager.has_template(SourcePlatform::Mijia, model.model.as_str());
        results.push(MiotDeviceModelResult {
            model,
            has_template,
        });
    }

    ok_data(results)
}



pub async fn templates(state: State<AppState>, Path(model): Path<String>) -> ApiResult<Vec<HlDeviceTemplate>> {
    let list = state.template_manager
        .templates
        .iter()
        .filter(|v| v.model.as_str() == model.as_str())
        .map(|v| v.clone())
        .collect::<Vec<HlDeviceTemplate>>();
    ok_data(list)
}

pub async fn convert_by_template(state: State<AppState>, Json(param): Json<MiConvertByTemplateParam>) -> ApiResult<()> {
    let mi_dev = MiotDeviceEntity::find_by_id(param.did.as_str())
        .one(state.conn())
        .await?
        .ok_or(api_err!("设备不存在"))?;
    state.template_manager.apply_template(state.conn(), &ApplyTemplateOptions {
        platform: SourcePlatformModel::MiHome(mi_dev),
        id: param.id,
        hap_manager: state.hap_manager.clone(),
        bridge_mode: param.bridge_mode,
        bridge_id: param.bridge_id,
        gateway_id: param.gateway_id,
    }).await.tap_err(|e| error!("转换错误:{:?}",e))?;

    ok_data(())
}

/// 转换
pub async fn access(state: State<AppState>, Json(param): Json<MiConvertToIotParam>) -> ApiResult<()> {
    let mi_dev = MiotDeviceEntity::find_by_id(param.did.as_str())
        .one(state.conn())
        .await?
        .ok_or(api_err!("设备不存在"))?;

    //创建iot设备
    let model = IotDeviceActiveModel {
        device_id: Set(SNOWFLAKE.next_id()),
        integration: Set(param.integration),
        name: Set(param.name),
        memo: Set(param.memo),
        disabled: Set(true),
        source_platform: Set(SourcePlatform::Mijia.as_ref().to_string()),
        source_id: Set(Some(param.did.clone())),
        device_type: Set(DeviceType::Normal),
        params: Set(JsonValue::Object(Default::default())),
        update_at: Set(DateTimeUtc::from(chrono::Local::now())),
        ..Default::default()
    };

    let dev = IotDeviceEntity::insert(model)
        .exec(state.conn())
        .await?;
    ok_data(())
}


