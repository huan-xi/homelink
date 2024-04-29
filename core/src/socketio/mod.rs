pub mod context;
pub mod task;

use std::sync::Arc;
use axum::handler::Handler;
use log::info;
use socketioxide::extract::{Data, State, SocketRef};
use socketioxide::layer::SocketIoLayer;
use serde_json::Value;
use socketioxide::socket::DisconnectReason;
use crate::api::state::AppState;
use crate::socketio::context::{SocketContext, SocketContextInner};
use crate::socketio::task::native_blt_log_task;
use crate::socketio::task::native_blt_log_task::NativeBltLogTask;


async fn on_connect(socket: SocketRef, Data(data): Data<Value>, ctx: State<SocketContext>) {
    info!("socket connected: {}", socket.id);
    info!("data: {:?}", data);
    info!("data: {:?}", data);

    socket.on("native_blt/unsub_log", |socket: SocketRef, context: State<SocketContext>| {
        context.remove_task(socket.id, native_blt_log_task::NAME);
    });
    socket.on("native_blt/sub_log", |socket: SocketRef, context: State<SocketContext>| {
        let socket = Arc::new(socket);
        tokio::spawn(async move {
            let task = NativeBltLogTask::new(socket.clone(), context.clone()).await;
            match task {
                Ok(task) => {
                    context.push_task(socket.id, Box::new(task));
                }
                Err(e) => {
                    info!("开启蓝牙日志失败: {:?}", e);
                }
            }
        });
    });

    socket.on_disconnect(|socket: SocketRef, reason: DisconnectReason, context: State<SocketContext>, | async move {
        info!("socket disconnected: {}, reason: {:?}", socket.id, reason);
        context.socket_tasks.remove(&socket.id);
    });

}

pub fn socket_io_layer(app: AppState) -> SocketIoLayer {
    let (layer, io) = socketioxide::SocketIo::builder()
        .with_state(Arc::new(SocketContextInner::new(app)))
        .build_layer();


    let (layer, io) = socketioxide::SocketIo::new_layer();

    io.ns("/", on_connect);

    // ServiceBuilder::new()
    //     .layer(CorsLayer::permissive()) // Enable CORS policy
    //     .layer(layer),
    layer
}

#[cfg(test)]
mod test {
    use axum::handler::Handler;
    use socketioxide::SocketIo;

    #[test]
    pub fn test() {
        /*       let (layer, io) = SocketIo::builder()
                   .with_state(UserCnt::new())
                   .build_layer();*/
    }
}