use std::sync::Arc;
use futures_util::future::BoxFuture;
use crate::device::gateway::gateway::OpenMiioGatewayDevice;
use crate::device::miot_spec_device::{BaseMiotSpecDevice, DeviceInfo, MiotSpecDevice};
use crate::device::emitter::DataListener;
use crate::proto::miio_proto::MiotSpecProtocolPointer;
use crate::proto::protocol::ExitError;
/// 通过米家网关的zigbee子设备

pub struct ZigbeeDevice {
    pub info: DeviceInfo,
    base: BaseMiotSpecDevice,
    gateway: Arc<OpenMiioGatewayDevice>,
}
impl MiotSpecDevice for ZigbeeDevice {

    fn get_info(&self) -> &DeviceInfo { &self.info }

    fn get_base(&self) -> &BaseMiotSpecDevice {
        &self.base
    }

    fn get_proto(&self) -> BoxFuture<Result<MiotSpecProtocolPointer, ExitError>> {
        self.gateway.get_proto() }

    fn run(&self) -> BoxFuture<Result<(), ExitError>> {
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

