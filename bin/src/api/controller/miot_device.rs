use std::time::Duration;

use anyhow::anyhow;
use axum::body::HttpBody;
use axum::extract::{Query, State};
use axum::Json;
use log::warn;
use sea_orm::{EntityTrait, PaginatorTrait};
use sea_orm::ActiveValue::Set;
use tokio::net::UdpSocket;

use miot_spec::proto::transport::udp_iot_spec_proto::UdpMiotSpecProtocol;

use crate::api::output::{ApiResult, err_msg, ok_data};
use crate::api::params::{AccountParam, DidParam, MiConvertToIotParam};
use crate::api::results::MiotDeviceResult;
use crate::api::state::AppState;
use crate::api_err;
use crate::db::entity::iot_device::{DeviceParam, IotDeviceType, SourceType};
use crate::db::entity::iot_device::DeviceParam::{MiGatewayParam, WifiDeviceParam};
use crate::db::entity::mi_account::MiAccountStatus;
use crate::db::entity::prelude::{IotDeviceActiveModel, IotDeviceEntity, MiAccountActiveModel, MiAccountEntity, MiAccountModel, MiotDeviceActiveModel, MiotDeviceEntity, MiotDeviceModel};
use crate::db::SNOWFLAKE;

// accounts
pub async fn update_account(state: State<AppState>, Json(param): Json<AccountParam>) -> ApiResult<()> {
    let count = MiAccountEntity::find_by_id(param.account.clone())
        .count(state.conn())
        .await?;
    if count > 0 {
        return err_msg("账号不存在");
    };
    let account = MiAccountActiveModel {
        account: Set(param.account),
        password: Set(param.password.ok_or(api_err!("密码不能为空"))?),
        ..Default::default()
    };
    MiAccountEntity::insert(account)
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

pub async fn list(state: State<AppState>) -> ApiResult<Vec<MiotDeviceModel>> {
    let list = MiotDeviceEntity::find()
        .all(state.conn())
        .await?;

    ok_data(list)
}

/// 转换
pub async fn convert_to_iot_device(state: State<AppState>, Json(param): Json<MiConvertToIotParam>) -> ApiResult<()> {
    let mi_dev = MiotDeviceEntity::find_by_id(param.did.as_str())
        .one(state.conn())
        .await?
        .ok_or(api_err!("设备不存在"))?;
    let dev_params = match param.device_type {
        IotDeviceType::MiWifiDevice => {
            if mi_dev.localip.is_none() {
                return err_msg("该设备无ip");
            }
            WifiDeviceParam
        }
        IotDeviceType::MiGatewayDevice => {
            if mi_dev.localip.is_none() {
                return err_msg("该设备无ip");
            }
            MiGatewayParam
        }
        IotDeviceType::MiMeshDevice => {
            if param.gateway_id.is_none() {
                return err_msg("网关id不能为空");
            }
            DeviceParam::MeshParam
        }
        _ => {
            return err_msg("暂不支持该设备类型");
        }
    };

    //创建iot设备
    let model = IotDeviceActiveModel {
        device_id: Set(SNOWFLAKE.next_id()),
        device_type: Set(param.device_type),
        params: Set(Some(dev_params)),
        gateway_id: Set(param.gateway_id),
        name: Set(Some(param.name)),
        memo: Default::default(),
        disabled: Set(false),
        source_type: Set(Some(SourceType::MiHome)),
        source_id: Set(Some(param.did.clone())),
    };

    let dev = IotDeviceEntity::insert(model)
        .exec(state.conn())
        .await?;
    ok_data(())
}


//删除账号

/*pub async fn delete_account(state: State<AppState>, Path(account): Path<String>) -> ApiResult<()> {
    MiAccountEntity::delete()
        .filter(MiAccountEntity::Account.eq(account))
        .exec(state.conn())
        .await?;
    ok_data(())
}*/

/*




/// 更新转换
pub fn update_iot_device() {}

/// 登入账号
pub async fn login_mi_account() {}

pub fn delete_device() {}*/