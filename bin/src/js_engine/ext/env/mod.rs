use std::cell::RefCell;
use std::rc::Rc;
use anyhow::anyhow;
use deno_runtime::deno_core;
use deno_runtime::deno_core::error::AnyError;
use deno_runtime::deno_core::OpState;
use derive_builder::Builder;
use sea_orm::DbConn;
use tokio::sync::oneshot;
use crate::init::device_manage::IotDeviceManager;
use crate::init::hap_manage::HapManage;

pub struct EnvContext {
    pub info: String,
    pub version: String,

    pub main_channel: Option<oneshot::Receiver<u8>>,
    pub conn: DbConn,
    pub dev_manager: IotDeviceManager,
    pub hap_manager: HapManage,
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
#[number]
pub async fn op_get_device(state: Rc<RefCell<OpState>>) -> Result<i64, AnyError> {
    // state.get_mut()
    Ok(1)
}


