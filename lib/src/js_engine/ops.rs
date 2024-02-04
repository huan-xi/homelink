use deno_runtime::deno_core;
use deno_runtime::deno_core::v8;
use crate::js_engine::context::{APP_JS_CONTEXT, get_app_js_context};

deno_core::extension!(get_iot_device_runtime, ops = [op_get_iot_device]);

pub async fn op_env_info(){

}

/// 获取设备
#[deno_core::op2(async)]
pub async fn op_get_iot_device(#[bigint] device_id: i64) {
    get_app_js_context().dev_manager.get_device(device_id);
    // let object = v8::Object::new();
}

// #[deno_core::op2(async)]

/// js ->get ch ->while(true) await ch.recv()

/// 等待接受 特征事件
/// 拿到设备的channel
/// 等待channel 的值, on_update->

pub async fn accept_mapping_characteristic_event(ch_id: i64) {
    // 从全局上下文中拿mapping_channel
    // let ch = MappingCharacteristicRecv::new();
    // ch.js_recv.recv().await;
}