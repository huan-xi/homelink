use std::collections::HashMap;
use axum::extract::State;
use axum::Json;
use crate::api::output::{ApiResult, ok_data};
use crate::api::params::LoginParam;
use crate::api::results::UserInfoResult;
use crate::api::state::AppState;

pub async fn info(state: State<AppState>) -> ApiResult<UserInfoResult> {
    let result = UserInfoResult {
        username: "admin".to_string(),
        roles: vec!["admin".to_string()],
    };
    ok_data(result)
}
pub async fn login(state: State<AppState>, Json(param): Json<LoginParam>) -> ApiResult<HashMap<String, String>> {
    let mut result = HashMap::new();
    result.insert("token".to_string(), "token".to_string());
    ok_data(result)
}