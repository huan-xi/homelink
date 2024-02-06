use std::sync::Arc;

use crate::device::gateway::gateway::OpenMiioGatewayDevice;
use crate::device::miot_spec_device::{AsMiotGatewayDevice, BaseMiotSpecDevice, DeviceInfo, MiotSpecDevice};
use crate::proto::miio_proto::MiotSpecProtocolPointer;
use crate::proto::protocol::ExitError;
/// 通过米家网关的zigbee子设备

pub struct ZigbeeDevice {
    pub info: DeviceInfo,
    base: BaseMiotSpecDevice,
    gateway: Arc<OpenMiioGatewayDevice>,
}

impl AsMiotGatewayDevice for ZigbeeDevice {

}

#[async_trait::async_trait]
impl MiotSpecDevice for ZigbeeDevice {

    fn get_info(&self) -> &DeviceInfo { &self.info }

    fn get_base(&self) -> &BaseMiotSpecDevice {
        &self.base
    }

    async fn get_proto(&self) -> Result<MiotSpecProtocolPointer, ExitError> {
        self.gateway.get_proto().await
    }

    async fn run(&self) -> Result<(), ExitError>{
        todo!()
    }
}
impl ZigbeeDevice {
    pub fn new(info: DeviceInfo, gateway: Arc<OpenMiioGatewayDevice>) -> ZigbeeDevice {
        ZigbeeDevice {
            info,
            base: Default::default(),
            gateway,
        }
    }
}

