use axum::extract::{Query, State};
use crate::api::output::{ApiResult, ok_data};
use crate::api::params::QueryIotDeviceParam;
use crate::api::results::IotDeviceResult;
use crate::api::state::AppState;
use crate::api_err;
use crate::db::entity::iot_device::SourcePlatform;

pub async fn list(state: State<AppState>, Query(param): Query<QueryIotDeviceParam>) -> ApiResult<Vec<IotDeviceResult>> {
    let source_platform = param.source_platform.clone()
        .ok_or(api_err!("source_platform is required"))?;
    match source_platform {
        SourcePlatform::Mijia => {
            //获取米家设备
        }
        SourcePlatform::BleNative => {
            todo!()
        }
    };
    ok_data(vec![])
}