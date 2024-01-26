use std::time::{SystemTime, UNIX_EPOCH};
use axum::extract::State;
use crate::api::output::{ApiResult, ok_data};
use crate::api::results::UserInfoResult;
use crate::api::state::AppState;
use crate::{api_err, err_msg};
use crate::config::context::get_app_context;
use crate::js_engine::channel::main_channel::{FromModuleResp, ToModuleEvent};
use crate::js_engine::channel::params::U64Value;

pub async fn ping_js(state: State<AppState>) -> ApiResult<String> {
    let context = get_app_context();
    let now = SystemTime::now();
    let time = now.duration_since(UNIX_EPOCH).unwrap().as_millis();
    let resp = context.js_engine.send(ToModuleEvent::Ping(U64Value::new(time))).await?;
    if let FromModuleResp::Pong(v) = resp {
        return ok_data(v.value);
    }

    return err_msg!("执行错误");
}