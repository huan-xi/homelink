use std::sync::Arc;
use futures_util::StreamExt;
use impl_new::New;
use log::info;
use socketioxide::extract::SocketRef;
use tokio::sync::oneshot;
use crate::socketio::context::{SocketContext, Task};
pub const NAME: &str = "NativeBltLogTask";
pub struct NativeBltLogTask {
    // socket: Arc<SocketRef>,
    context: SocketContext,
    ctrl: Option<oneshot::Sender<()>>,
}

impl Drop for NativeBltLogTask {
    fn drop(&mut self) {
        if let Some(s)= self.ctrl.take() {
            info!("drop NativeBltLogTask");
            s.send(()).ok();
        }
    }
}

impl NativeBltLogTask {
    pub async fn new(socket: Arc<SocketRef>, context: SocketContext) -> anyhow::Result<Self> {
        let (ctrl, rx) = oneshot::channel::<()>();
        let mut listener = context.app.ble_manager.adapter_event_listener().await?;
        tokio::spawn(async move {
            let task = async move {
                // let mut events = event.recv().await;
                //发送数据
                while let Some(event) = listener.next().await {
                    if let Err(e) = socket.emit("native_blt_log", event) {
                        log::error!("send native_blt_log error: {:?}", e);
                        break;
                    }


                }
            };

            loop {
                tokio::select! {
                    _ = rx => {
                        break;
                    }
                    _ = task => {
                        //发送数据
                        break;
                    }
                }
            }
        });

        Ok(Self {
            // socket,
            ctrl: Some(ctrl),
            context,
        })
    }
}

impl Task for NativeBltLogTask {
    fn name(&self) -> String {
        NAME.to_string()
    }
}