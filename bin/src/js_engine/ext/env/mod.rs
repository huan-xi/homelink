use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use anyhow::anyhow;
use dashmap::DashMap;
use deno_runtime::deno_core;
use deno_runtime::deno_core::error::AnyError;
use deno_runtime::deno_core::{op2, OpState, ResourceId};
use sea_orm::{DbConn, JsonValue};
use tokio::sync::oneshot;
use crate::init::device_manage::IotDeviceManager;
use crate::init::hap_manage::HapManage;
use crate::js_engine::channel::{main_channel, MsgId};
use crate::js_engine::channel::main_channel::{ReceiverResource, ReceiverResult, ToModuleEvent};


pub struct EnvContext {
    pub info: String,
    pub version: String,
    pub main_recv: Option<main_channel::ModuleRecv>,
    pub conn: DbConn,
    pub dev_manager: IotDeviceManager,
    pub hap_manager: HapManage,
    // 特征与模块通道的映射
    // 和context_js中的map 是同一个,
    // pub mapping_characteristic_map: Arc<DashMap<i64, MappingCharacteristicRecv>>,
    // pub hap_module_map: HapAccessoryModuleMapPointer,
}






deno_core::extension!(deno_env,
    deps = [ deno_webidl, deno_console ],
    ops = [
        op_open_main_listener,
        op_update_char_value,
        op_device_set_property,
        op_device_read_property,
        op_send_resp,
        op_accept_event
    ],
    esm_entry_point = "ext:deno_env/env.js",
    esm = [ dir "src/js_engine/ext/env","env.js" ],
    options = {
    env_context: EnvContext
    },
    state = |state, options| {
      state.put( options.env_context );
    },
);


#[deno_core::op2(async)]
#[smi]
pub async fn op_open_main_listener(state: Rc<RefCell<OpState>>) -> Result<ResourceId, AnyError> {
    let res = state.borrow_mut()
        .borrow_mut::<EnvContext>()
        .main_recv
        .as_mut()
        .ok_or(anyhow!("无主通道接收器"))?
        .take_receiver_resource()?;

    let rid = state.borrow_mut()
        .resource_table
        .add(res);
    Ok(rid)
}

#[deno_core::op2(async)]
pub async fn op_send_resp(state: Rc<RefCell<OpState>>,
                          #[bigint] msg_id: u64, #[serde] resp: main_channel::FromModuleResp) -> Result<(), AnyError> {
    let sender = state.borrow_mut()
        .borrow_mut::<EnvContext>()
        .main_recv
        .as_mut()
        .ok_or(anyhow!("无主通道接收器"))?
        .result_sender.clone();
    sender.send((msg_id, resp))?;

    Ok(())
}

#[deno_core::op2(async)]
#[serde]
pub async fn op_accept_event(state: Rc<RefCell<OpState>>,
                             #[smi] rid: ResourceId) -> Result<Option<ReceiverResult>, AnyError> {
    let mut recv = state
        .borrow_mut()
        .resource_table
        .get::<ReceiverResource>(rid)?;
    Ok(recv.recv().await)
}


#[deno_core::op2(async)]
pub async fn op_device_set_property(state: Rc<RefCell<OpState>>,
                                    #[bigint] device_id: i64,
                                    siid: i32,
                                    piid: i32,
                                    #[serde] value: JsonValue) -> Result<(), AnyError> {
    let mut dev = state.borrow()
        .borrow::<EnvContext>()
        .dev_manager
        .get_device(device_id)
        .ok_or(anyhow!("设备不存在"))?;
    dev.set_property(siid, piid, value).await?;
    Ok(())
}

/// 设备读取属性
#[deno_core::op2(async)]
#[serde]
pub async fn op_device_read_property(state: Rc<RefCell<OpState>>,
                                     #[bigint] device_id: i64,
                                     siid: i32,
                                     piid: i32) -> Result<Option<JsonValue>, AnyError> {
    let mut dev = state.borrow()
        .borrow::<EnvContext>()
        .dev_manager
        .get_device(device_id)
        .ok_or(anyhow!("设备不存在"))?;
    let val = dev.read_property(siid, piid).await?;
    Ok(val)
}

/// 更新特征值
#[deno_core::op2(async)]
pub async fn op_update_char_value(state: Rc<RefCell<OpState>>,
                                  #[bigint] aid: u64,
                                  #[string] service_tag: String,
                                  #[string] char_tag: String,
                                  #[serde]  json_value: JsonValue) -> anyhow::Result<()>{
    let mut hap = state.borrow()
        .borrow::<EnvContext>()
        .hap_manager.clone();
    hap.update_char_value(aid,service_tag,char_tag,json_value).await
}