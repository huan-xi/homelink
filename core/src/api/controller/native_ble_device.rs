use axum::extract::{Query, State};
use btleplug::api::{Central, Peripheral};
use crate::api::output::{ApiResult, ok_data};
use crate::api::params::QueryIotDeviceParam;
use crate::api::results::{NativeBleDevice, NativeBleStatus};
use crate::api::state::AppState;
use crate::init::manager::ble_manager::Status;


///监听消息


pub async fn status(state: State<AppState>, Query(param): Query<QueryIotDeviceParam>) -> ApiResult<NativeBleStatus> {
    let status = state.ble_manager.status.read().await.clone();
    ok_data(NativeBleStatus{
        status,
    })
}
pub async fn list(state: State<AppState>, Query(param): Query<QueryIotDeviceParam>) -> ApiResult<Vec<NativeBleDevice>> {
    let status = state.ble_manager.status.read().await.clone();
    let mut peripherals = vec![];
    if status == Status::On {
        if let Some(adapter) = state.ble_manager.adapter.read().await.as_ref() {
            let mut ps = adapter.peripherals().await?;
            for p in ps {
                let prop = p.properties().await?;
                let addr = p.address();

                peripherals.push(NativeBleDevice {
                    id: p.id().to_string(),
                    mac: format!("{:x}", addr),
                    name: prop.as_ref().and_then(|p| p.local_name.clone()),
                    rssi: prop.as_ref().and_then(|p| p.rssi),
                });
            }
        }
    };
    ok_data(peripherals)
}