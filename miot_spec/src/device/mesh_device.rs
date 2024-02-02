use std::time::Duration;
use crate::device::common::emitter::EventType;

use crate::device::miot_spec_device::{BaseMiotSpecDevice, DeviceInfo, MiotSpecDevice};
use crate::device::MiotDevicePointer;
use crate::proto::miio_proto::{MiotSpecDTO, MiotSpecId, MiotSpecProtocolPointer};
use crate::proto::protocol::{ExitError, RecvMessage};

/// 网关上监听了米家协议
///能监听到  miio/report properties_changed
// mesh 设备
pub struct MeshDevice {
    info: DeviceInfo,
    base: BaseMiotSpecDevice,
    gateway: MiotDevicePointer,
}

#[async_trait::async_trait]
impl MiotSpecDevice for MeshDevice {
    fn get_info(&self) -> &DeviceInfo { &self.info }
    fn get_base(&self) -> &BaseMiotSpecDevice {
        &self.base
    }
    async fn get_proto(&self) -> Result<MiotSpecProtocolPointer, ExitError> {
        self.gateway.get_proto().await
    }

    async fn run(&self) -> Result<(), ExitError> {
        let mut recv = self.gateway.get_event_recv().await;
        //{"id":1996772508,"method":"properties_changed","params":[{"did":"1023054714","siid":2,"piid":1,"value":false}],"type":16}
        while let Ok(EventType::GatewayMsg(msg)) = recv.recv().await {
            let mut data = msg.data;
            if let Some(true) = data
                .remove("method")
                .map(|i| i.as_str() == Some("properties_changed")) {
                if let Some(mut params) = data.remove("params") {
                    if let Some(params) = params.as_array_mut() {
                        let mut updates = vec![];
                        for param in params {
                            if let Some(param) = param.as_object_mut() {
                                if Some(true) == param.remove("did")
                                    .map(|f| f.as_str() == Some(self.info.did.as_str())) {
                                    //判断属性
                                    let siid = param.remove("siid").and_then(|f| f.as_i64());
                                    let piid = param.remove("piid").and_then(|f| f.as_i64());
                                    let value = param.remove("value");
                                    if let (Some(siid), Some(piid), Some(value)) = (siid, piid, value) {
                                        let id = MiotSpecId::new(siid as i32, piid as i32);
                                        //更新属性
                                        self.base.value_map.write().await.insert(id, value.clone());
                                        let result = MiotSpecDTO {
                                            did: self.info.did.clone(),
                                            siid: siid as i32,
                                            piid: piid as i32,
                                            value: Some(value),
                                        };
                                        self.base.emitter.write().await.emit(EventType::UpdateProperty(result.clone())).await;
                                        updates.push(result);
                                    }
                                }
                            }
                        }
                        if !updates.is_empty() {
                            self.base.emitter.write().await.emit(EventType::UpdatePropertyBatch(updates)).await;
                        }
                    };
                }
            }
        }
        Ok(())
    }
}

impl MeshDevice {
    pub fn new(info: DeviceInfo, gateway: MiotDevicePointer) -> MeshDevice {
        MeshDevice {
            info,
            base: Default::default(),
            gateway,
        }
    }
}