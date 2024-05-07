use axum::extract::State;
use crate::api::output::{ApiResult, ok_data};
use crate::api::state::AppState;

pub async fn restart(state: State<AppState>) -> ApiResult<()> {
    if let Some(s) = state.server_shutdown_signal.lock().await.take() {
        s.shutdown().await;
    }
    //todo 开启脚本重新运行当前进程

    ok_data(())
}