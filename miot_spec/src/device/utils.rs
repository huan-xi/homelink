use std::time::Duration;
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use log::error;
use tokio::time;
use crate::device::emitter::EventType;
use crate::device::miot_spec_device::{BaseMiotSpecDevice, MiotSpecDevice};
use crate::proto::miio_proto::MiotSpecDTO;

/// 轮询注册的属性,发送UpdateProperty 事件
pub fn get_poll_func<'a, T: MiotSpecDevice + Sync + Send>(dev: &'a T, did: &'a str, interval: Duration) -> BoxFuture<'a, ()> {
    let mut interval = time::interval(interval);

    async move {
        loop {
            interval.tick().await;
            // tokio::time::sleep(interval).await;
            let base = dev.get_base();
            if base.emitter.read().await.is_empty() {
                continue;
            };
            let proto = match dev.get_proto().await {
                Ok(p) => {
                    p
                }
                Err(e) => {
                    error!("获取协议失败");
                    break;
                }
            };

            let params = base
                .registered_property
                .read()
                .await.iter()
                .map(|id| MiotSpecDTO {
                    did: did.to_string(),
                    siid: id.siid,
                    piid: id.piid,
                    value: None,
                }).collect::<Vec<MiotSpecDTO>>();
            if params.is_empty() {
                continue;
            }
            if let Ok(results) = proto.get_properties(params, None).await {
                for result in results {
                    base.emitter.clone().write().await.emit(EventType::UpdateProperty(result)).await;
                }
            }
        }
    }.boxed()
}