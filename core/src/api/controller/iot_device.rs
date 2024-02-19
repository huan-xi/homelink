use axum::extract::{Path, Query, State};
use axum::Json;
use sea_orm::ActiveValue::Set;
use sea_orm::{EntityTrait, JsonValue, PaginatorTrait};
use crate::api::output::{ApiResp, ApiResult, ok_data};
use crate::api::state::AppState;
use crate::db::entity::prelude::{IotDeviceEntity, IotDeviceActiveModel, IotDeviceColumn, IotDeviceModel, MiotDeviceEntity, MiotDeviceModel, HapAccessoryEntity, HapAccessoryColumn};
use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;
use miot_proto::device::miot_spec_device::{AsMiotDevice, MiotDeviceArc};
use miot_proto::proto::miio_proto::MiotSpecId;
use crate::api::params::{AddServiceParam, DisableParam, QueryIotDeviceParam, TestPropParam};
use crate::api::results::{IotDeviceResult, MiotDeviceResult};
use crate::api_err;
use crate::db::entity::iot_device::SourcePlatform;

pub async fn list(state: State<AppState>, Query(param): Query<QueryIotDeviceParam>) -> ApiResult<Vec<IotDeviceResult>> {
    let mut query = IotDeviceEntity::find();
    if let Some(f) = param.device_type {
        query.query().and_where(IotDeviceColumn::DeviceType.eq(f));
    };
    let list = query
        .all(state.conn())
        .await?;

    let dev_manager = state.device_manager.clone();
    let mut result_list: Vec<IotDeviceResult> = vec![];
    for i in list.into_iter() {
        let running = dev_manager.is_running(i.device_id);
        //查询对应的来源设
        let source = if let (st, Some(id)) = (i.source_platform, i.source_id.clone()) {
            match st {
                SourcePlatform::MiHome => {
                    //查询对应的设备
                    MiotDeviceEntity::find_by_id(id).one(state.conn()).await?
                }
                _ => {
                    None
                }
            }
        } else {
            None
        };
        // get_source_device(&i)?;
        let dev = match source {
            None => None,
            Some(s) => {
                Some(serde_json::from_value::<MiotDeviceResult>(s.full)?)
            }
        };
        result_list.push(IotDeviceResult {
            model: i,
            running,
            source: dev,
        })
    }

    Ok(ApiResp::with_data(result_list))
}


pub async fn read_property(state: State<AppState>, Path(id): Path<i64>, Json(params): Json<TestPropParam>) -> ApiResult<Option<JsonValue>> {
   let dev = state.device_manager
       .get_device(id)
        .ok_or(api_err!("设备不存在"))?;
    let dev = MiotDeviceArc(dev);
    let value = dev.as_miot_device()?.read_property(params.siid, params.piid).await?;
    Ok(ApiResp::with_data(value))

}

pub async fn set_property(state: State<AppState>, Path(id): Path<i64>, Json(params): Json<TestPropParam>) -> ApiResult<()> {
 let dev = state.device_manager
        .get_device(id)
        .ok_or(api_err!("设备不存在"))?;
    let val = params.value
        .ok_or(api_err!("请设置value"))?;
    let dev = MiotDeviceArc(dev);
    dev.as_miot_device()?.set_property(MiotSpecId::new(params.siid, params.piid), val)
        .await?;

    ok_data(())

}

pub async fn delete(state: State<AppState>, Path(id): Path<i64>) -> ApiResult<()> {
    //查询配件
    let count = HapAccessoryEntity::find()
        .filter(HapAccessoryColumn::DeviceId.eq(id))
        .count(state.conn())
        .await?;
    if count > 0 {
        return Err(api_err!("设备下有配件,请先删除配件"));
    }
    IotDeviceEntity::delete_by_id(id).exec(state.conn()).await?;
    Ok(ApiResp::with_data(()))
}


pub async fn restart(state: State<AppState>, Path(id): Path<i64>) -> ApiResult<()> {
    state.device_manager.stop_device(id)?;
    let model = IotDeviceEntity::find_by_id(id).one(state.conn()).await?;
    let iot_device = model.ok_or(api_err!("设备不存在"))?;
   /* match iot_device.device_type.require_gw() {
        true => {
            init_children_device(state.conn(), iot_device, state.device_manager.clone())
                .await
                .map_err(|e| api_err!("启动失败{e}"))?
        }
        false => {
            init_mi_device(state.conn(), iot_device, state.device_manager.clone(), state.mi_account_manager.clone())
                .await
                .map_err(|e| api_err!("启动失败{e}"))?
        }
    };*/

    ok_data(())
}


pub async fn disable(state: State<AppState>, Path(id): Path<i64>, Query(param): Query<DisableParam>) -> ApiResult<()> {
    let model = IotDeviceActiveModel {
        device_id: Set(id),
        disabled: Set(param.disabled),
        ..Default::default()
    };
    IotDeviceEntity::update(model)
        .filter(IotDeviceColumn::DeviceId.eq(id))
        .exec(state.conn()).await?;
    Ok(ApiResp::with_data(()))
}


/// 导出模板

pub async fn export_template(state: State<AppState>, Path(id): Path<i64>) {}