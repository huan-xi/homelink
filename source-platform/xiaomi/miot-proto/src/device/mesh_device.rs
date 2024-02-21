use std::sync::Arc;
use crate::device::common::emitter::MijiaEvent;

use crate::device::miot_spec_device::{AsMiotDevice,  BaseMiotSpecDevice, DeviceInfo, MiotDeviceType, MiotSpecDevice, MiotSpecDeviceWrapper};
use crate::proto::miio_proto::{MiotSpecDTO, MiotSpecId, MiotSpecProtocolPointer};
use crate::proto::protocol::{ExitError};


pub type MeshDevice = MiotSpecDeviceWrapper;

/// 网关上监听了米家协议
///能监听到  miio/report properties_changed
// mesh 设备
pub struct MeshDeviceInner<T: AsMiotDevice> {
    info: DeviceInfo,
    base: BaseMiotSpecDevice,
    gateway: T,
}


#[async_trait::async_trait]
impl<T: AsMiotDevice> MiotSpecDevice for MeshDeviceInner<T> {
    fn get_info(&self) -> &DeviceInfo { &self.info }
    fn get_base(&self) -> &BaseMiotSpecDevice {
        &self.base
    }
    async fn get_proto(&self) -> Result<MiotSpecProtocolPointer, ExitError> {
        self.gateway
            .as_miot_device()?.get_proto().await
    }

    async fn run(&self) -> Result<(), ExitError> {
        let mut recv = self.gateway
            .as_miot_device()?
            .get_event_recv().await;
        //{"id":1996772508,"method":"properties_changed","params":[{"did":"1023054714","siid":2,"piid":1,"value":false}],"type":16}
        while let Ok(MijiaEvent::GatewayMsg(msg)) = recv.recv().await {
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
                                        // self.base.emitter.write().await.emit(MijiaEvent::UpdateProperty(result.clone())).await;
                                        updates.push(result);
                                    }
                                }
                            }
                        }
                        if !updates.is_empty() {
                            let event =Arc::new( MijiaEvent::PropertiesChanged(updates));
                            self.base.emitter.emit(event).await;
                        }
                    };
                }
            }
        }
        Ok(())
    }
}

impl MeshDevice {
    pub fn new_mesh_device<T: AsMiotDevice+ 'static>(info: DeviceInfo, gateway: T) -> MeshDevice {
        let inner = MeshDeviceInner {
            info,
            base: Default::default(),
            gateway,
        };
        MiotSpecDeviceWrapper(Box::new(inner), MiotDeviceType::Mesh)
    }
}