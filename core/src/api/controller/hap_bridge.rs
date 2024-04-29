use axum::extract::{Path, Query, State};
use axum::Json;
use sea_orm::{ActiveEnum, ConnectionTrait, EntityTrait, IntoActiveModel, ModelTrait};
use sea_orm::ActiveValue::Set;
use sea_orm::prelude::Expr;
use hap::BonjourStatusFlag;
use sea_orm::PaginatorTrait;
use sea_orm::*;
use crate::{api_err, err_msg};
use crate::api::output::{ApiResp, ApiResult, ok_data};
use crate::api::params::{AddHapBridgeParam, DisableParam, GetTemplateParam};
use crate::api::results::{HapBridgeResult, TemplateResult};
use crate::api::state::AppState;
use crate::db::entity::prelude::{HapAccessoryColumn, HapAccessoryEntity, HapBridgeActiveModel, HapBridgeColumn, HapBridgeEntity, HapBridgeModel};
use sea_orm::QueryFilter;
use crate::api::params::power::power_query_param::PowerQueryParam;
use crate::api::params::power::power_update_param::PowerUpdateParam;
use crate::db::entity::hap_bridge::{BonjourStatusFlagWrapper, BridgeInfo, Model, PairingsWrapper};
use crate::db::service::hap_bridge_service::create_hap_bridge;
use crate::db::SNOWFLAKE;
use crate::init::hap_init::add_hap_bridge;
use crate::init::manager::device_manager::IotDeviceManager;
use crate::service::hap_bridge_service;
use crate::service::hap_bridge_service::to_model_result;
use crate::template::hap::bridge::HapBridgeTemplate;

pub async fn update_by_template(state: State<AppState>, Json(param): Json<TemplateResult>) -> ApiResult<()>{
    let template: HapBridgeTemplate = param.format.parse(param.text.as_str())?;
    let mut model = template.try_into_update_model()?;
    model.update(state.conn()).await?;
    ok_data(())
}

pub async fn add_by_template(state: State<AppState>, Json(param): Json<TemplateResult>) -> ApiResult<()> {
    let template: HapBridgeTemplate = param.format.parse(param.text.as_str())?;
    let name = template.name.clone().ok_or(api_err!("名称不能为空"))?;
    let pin_code = template.pin_code.clone().ok_or(api_err!("pin_code不能为空"))?;

    hap_bridge_service::add_check(&state, name.as_str(), pin_code).await?;

    let mut model = template.try_into_insert_model()?;
    let id = SNOWFLAKE.next_id();
    model.bridge_id = Set(id);

    let _ = HapBridgeEntity::insert(model).exec(state.conn()).await?;
    let hap_bridge = HapBridgeEntity::find()
        .filter(HapBridgeColumn::BridgeId.eq(id))
        .one(state.conn())
        .await?.ok_or(api_err!("添加失败"))?;
    add_hap_bridge(state.conn(), hap_bridge, state.hap_manager.clone(), state.device_manager.clone())
        .await
        .map_err(|e| api_err!("添加成功,启动失败{e}"))?;

    ok_data(())
}

/// 添加桥接器
pub async fn update(state: State<AppState>, Json(param): Json<PowerUpdateParam>) -> ApiResult<()> {
    let mut model = param.to_active_model::<HapBridgeEntity, HapBridgeActiveModel>()?;
    model.update_at = Set(chrono::Local::now().naive_local());
    model.update(state.conn()).await?;
    ok_data(())
}
pub async fn add(state: State<AppState>, Json(param): Json<AddHapBridgeParam>) -> ApiResult<()> {
    //查询名字是否存在
    let hap_bridge = create_hap_bridge(state.conn(), param.pin_code, param.category, param.name, false).await?;

    add_hap_bridge(state.conn(), hap_bridge, state.hap_manager.clone(), state.device_manager.clone())
        .await
        .map_err(|e| api_err!("添加成功,启动失败{e}"))?;
    ok_data(())
}


pub async fn delete(state: State<AppState>, Path(id): Path<i64>) -> ApiResult<()> {
    //查询配件数量
    let accessory_count = HapAccessoryEntity::find()
        .filter(HapAccessoryColumn::BridgeId.eq(id))
        .count(state.conn())
        .await?;
    if accessory_count > 0 {
        return err_msg!("请先删除桥接器下的配件");
    }
    state.hap_manager.stop_server(id).await?;
    HapBridgeEntity::delete_by_id(id).exec(state.conn()).await?;
    ok_data(())
}

///重置
pub async fn reset(state: State<AppState>, Path(id): Path<i64>) -> ApiResult<()> {
    let model = HapBridgeActiveModel {
        bridge_id: Set(id),
        pairings: Set(PairingsWrapper::default()),
        status_flag: Set(BonjourStatusFlagWrapper(BonjourStatusFlag::NotPaired)),
        ..Default::default()
    };
    HapBridgeEntity::update(model).exec(state.conn()).await?;
    restart(state, Path(id)).await?;
    ok_data(())
}

/// 重启桥接器
pub async fn restart(state: State<AppState>, Path(id): Path<i64>) -> ApiResult<()> {
    state.hap_manager.stop_server(id).await?;
    let model = HapBridgeEntity::find_by_id(id).one(state.conn()).await?;
    let hap_bridge = model.ok_or(api_err!("桥接器不存在"))?;
    add_hap_bridge(state.conn(), hap_bridge, state.hap_manager.clone(), state.device_manager.clone())
        .await
        .map_err(|e| api_err!("停止成功,启动失败{e}"))?;
    ok_data(())
}

///获取桥接器模板
pub async fn get_template(state: State<AppState>, Path(id): Path<i64>, Query(param): Query<GetTemplateParam>) -> ApiResult<TemplateResult> {
    let model = HapBridgeEntity::find_by_id(id).one(state.conn()).await?;
    let hap_bridge = model.ok_or(api_err!("桥接器不存在"))?;
    let template = HapBridgeTemplate::try_from(hap_bridge)?;
    let text = param.format.format_to_str(&template)?;
    ok_data(TemplateResult {
        text,
        format: param.format,
    })
}

pub async fn get_detail(state: State<AppState>, Path(id): Path<i64>) -> ApiResult<HapBridgeResult> {
    let model = HapBridgeEntity::find_by_id(id).one(state.conn()).await?;
    let hap_bridge = model.ok_or(api_err!("桥接器不存在"))?;
    let result = to_model_result(&state, &state.hap_manager, hap_bridge).await?;
    ok_data(result)
}

pub async fn list(state: State<AppState>, Query(param): Query<PowerQueryParam>) -> ApiResult<Vec<HapBridgeResult>> {
    let condition = param.get_condition::<HapBridgeEntity>()?;

    let list = HapBridgeEntity::find()
        .all(state.conn())
        .await?;

    let manager = state.hap_manager.clone();
    let mut r_list = vec![];
    for i in list {
        let result = to_model_result(&state, &manager, i).await?;
        r_list.push(result);
    }
    Ok(ApiResp::with_data(r_list))
}


pub async fn disable(state: State<AppState>, Path(id): Path<i64>, Query(param): Query<DisableParam>) -> ApiResult<()> {
    let model = HapBridgeEntity::find_by_id(id).one(state.conn()).await?;
    let mut hap_bridge = model.ok_or(api_err!("桥接器不存在"))?;
    hap_bridge.disabled = param.disabled;

    let update_model = HapBridgeActiveModel {
        bridge_id: Set(id),
        disabled: Set(param.disabled),
        ..Default::default()
    };

    HapBridgeEntity::update(update_model).exec(state.conn()).await?;
    if param.disabled {
        state.hap_manager.stop_server(id).await?;
    } else {
        add_hap_bridge(state.conn(), hap_bridge, state.hap_manager.clone(), state.device_manager.clone())
            .await
            .map_err(|e| api_err!("停止成功,启动失败{e}")).unwrap();
    }
    ok_data(())
}

