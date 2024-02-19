use std::time::Duration;
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use log::error;
use tokio::time;
use hl_integration::platform::hap::hap_device;
use crate::device::common::emitter::EventType;
use crate::device::miot_spec_device::{MiotSpecDevice};
use crate::proto::miio_proto::{MiotSpecDTO, MiotSpecId};
use crate::proto::protocol::ExitError;


pub async fn poll0<'a, T: MiotSpecDevice + Sync + Send>(dev: &'a T, did: &'a str) -> Result<(), ExitError> {
    let base = dev.get_base();
    if base.emitter.read().await.is_empty() {
        return Ok(());
    };
    let proto = dev.get_proto().await?;

    let params = base
        .poll_properties
        .read()
        .await.iter()
        .map(|id| MiotSpecDTO {
            did: did.to_string(),
            siid: id.siid,
            piid: id.piid,
            value: None,
        }).collect::<Vec<MiotSpecDTO>>();
    if params.is_empty() {
        return Ok(());
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


    Ok(())
}

/// 轮询注册的属性,发送UpdateProperty 事件
pub fn get_poll_func<'a, T: MiotSpecDevice + Sync + Send>(dev: &'a T, did: &'a str, interval: Duration) -> BoxFuture<'a, ()> {
    let mut interval = time::interval(interval);

    async move {
        loop {
            if let Err(e) = poll0(dev, did).await {
                error!("获取协议失败:{:?}",e);
                break;
            }
            interval.tick().await;
        }
    }.boxed()
}

pub fn get_hap_device_info(dev: &dyn  MiotSpecDevice) -> hap_device::DeviceInfo {
    let info = dev.get_info();

    let firmware_revision = info
        .extra.as_ref()
        .and_then(|i| i.fw_version.clone());
    let parts: Vec<&str> = info.model.split('.').collect();
    let manufacturer = parts.first()
        .map(|f| f.to_string())
        .unwrap_or("未知制造商".to_string());
    hap_device::DeviceInfo {
        manufacturer,
        model: info.model.clone(),
        serial_number: info.did.clone(),
        software_revision: None,
        firmware_revision,
    }
}