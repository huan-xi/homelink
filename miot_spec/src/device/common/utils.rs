use std::time::Duration;
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use log::error;
use tokio::time;
use crate::device::common::emitter::EventType;
use crate::device::miot_spec_device::{BaseMiotSpecDevice, MiotSpecDevice};
use crate::proto::miio_proto::{MiotSpecDTO, MiotSpecId};

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
                .pool_properties
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
                let mut updates = vec![];
                for result in results {
                    //新旧值比较
                    let id = MiotSpecId::new(result.siid, result.piid);

                    if let Some(old_val) = base.value_map.read().await.get(&id) {
                        if let Some(val) = &result.value {
                            if val == old_val {
                                continue;
                            } else {
                                base.value_map.write().await.insert(id, val.clone());
                                //发布单次更新事件
                                base.emitter.write().await.emit(EventType::UpdateProperty(result.clone())).await;
                                updates.push(result);
                            }
                        }
                    }
                }
                //发布批量更新事件
                if !updates.is_empty() {
                    base.emitter.write().await.emit(EventType::UpdatePropertyBatch(updates)).await;
                }
            }
        }
    }.boxed()
}