use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use anyhow::anyhow;
use dashmap::DashMap;
use deno_runtime::deno_core;
use deno_runtime::deno_core::error::AnyError;
use deno_runtime::deno_core::{op2, OpState, ResourceId};
use sea_orm::DbConn;
use tokio::sync::oneshot;
use crate::init::device_manage::IotDeviceManager;
use crate::init::hap_manage::HapManage;
use crate::js_engine::channel::hap_channel::{HapAccessoryModuleRecv, ReceiverResource, ToModuleEvent};
use crate::js_engine::channel::mapping_characteristic_channel::{MappingCharacteristicRecv, MappingCharacteristicSender};
use crate::js_engine::init_js_engine::MsgId;


pub struct EnvContext {
    pub info: String,
    pub version: String,
    pub main_channel: Option<oneshot::Receiver<u8>>,
    pub conn: DbConn,
    pub dev_manager: IotDeviceManager,
    pub hap_manager: HapManage,
    /// 特征与模块通道的映射
    /// 和context_js中的map 是同一个,
    pub mapping_characteristic_map: Arc<DashMap<i64, MappingCharacteristicRecv>>,
    pub hap_module_map: Arc<DashMap<i64, HapAccessoryModuleRecv>>,
}






deno_core::extension!(deno_env,
    deps = [ deno_webidl, deno_console ],
    ops = [op_get_device,op_main_listen],
    esm_entry_point = "ext:deno_env/01_env.js",
    esm = [ dir "src/js_engine/ext/env","01_env.js" ],
    options = {
    env_context: EnvContext
    },
    state = |state, options| {
      state.put( options.env_context );
    },
);


#[deno_core::op2(async)]
#[number]
pub async fn op_main_listen(state: Rc<RefCell<OpState>>) -> Result<u64, AnyError> {
    let main_channel = state.borrow_mut().borrow_mut::<EnvContext>()
        .main_channel.take()
        .ok_or(anyhow!("main channel为空"))?;
    let code = main_channel.await? as u64;
    Ok(code)
}

#[deno_core::op2(async)]
#[smi]
pub async fn open_hap_listener(state: Rc<RefCell<OpState>>, #[number] aid: i64) -> Result<ResourceId, AnyError> {
    let res = state.borrow()
        .borrow::<EnvContext>().hap_module_map
        .get_mut(&aid)
        .ok_or(anyhow!("没有该设备运行通道"))?
        .take_receiver_resource()?;

    let rid = state.borrow_mut()
        .resource_table
        .add(res);
    Ok(rid)
}

#[deno_core::op2(async)]
#[serde]
pub async fn accept_event(state: Rc<RefCell<OpState>>,
                          #[smi] rid: ResourceId, ) -> Result<Option<(MsgId,ToModuleEvent)>, AnyError> {
    let recv = state
        .borrow_mut()
        .resource_table
        .take::<ReceiverResource>(rid)?;
    let recv = Rc::try_unwrap(recv)
        .ok()
        .expect("multiple op_fetch_send ongoing");
    Ok(recv.0.await)
}

#[deno_core::op2(async)]
#[number]
pub async fn op_get_device(state: Rc<RefCell<OpState>>) -> Result<i64, AnyError> {
    // state.get_mut()
    Ok(1)
}


