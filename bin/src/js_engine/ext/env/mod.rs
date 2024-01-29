use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use anyhow::anyhow;
use deno_runtime::deno_core;
use deno_runtime::deno_core::{OpState, ResourceId};
use deno_runtime::deno_core::error::AnyError;
use sea_orm::JsonValue;
use tokio::time::timeout;

use crate::js_engine::channel::main_channel;
use crate::js_engine::channel::main_channel::{ReceiverResource, ReceiverResult};
use crate::js_engine::context::EnvContext;

deno_core::extension!(deno_env,
    deps = [ deno_webidl, deno_console ],
    ops = [
        op_open_main_listener,
        op_update_char_value,
        op_device_set_property,
        op_device_read_property,
        op_send_resp,
        op_accept_event,
        op_mock_err
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
pub async fn test() -> anyhow::Result<Option<JsonValue>> {
    timeout(Duration::from_secs(1), async {
        tokio::time::sleep(Duration::from_secs(20)).await;
        Ok(Some(JsonValue::from(1)))
    }).await.map_err(|f| anyhow!("执行超时"))?
}

#[deno_core::op2(async)]
#[serde]
pub async fn op_mock_err(state: Rc<RefCell<OpState>>) -> Result<Option<JsonValue>, AnyError> {
    let mut dev = state.borrow()
        .borrow::<EnvContext>()
        .dev_manager
        .get_device(3)
        .ok_or(anyhow!("设备不存在"))?;
    let a = test().await?;
    Ok(a)
}

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
    dev.set_property(siid, piid, value).await
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
    dev.read_property(siid, piid).await
}

/// 更新特征值
#[deno_core::op2(async)]
pub async fn op_update_char_value(state: Rc<RefCell<OpState>>,
                                  #[bigint] aid: u64,
                                  #[string] service_tag: String,
                                  #[string] char_tag: String,
                                  #[serde]  json_value: JsonValue) -> anyhow::Result<()> {
    let mut hap = state.borrow()
        .borrow::<EnvContext>()
        .hap_manager.clone();
    hap.update_char_value(aid, service_tag, char_tag, json_value).await
}