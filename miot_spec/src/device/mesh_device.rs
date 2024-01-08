use std::sync::Arc;
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use crate::device::gateway::gateway::OpenMiioGatewayDevice;
use crate::device::miot_spec_device::{DeviceInfo, MiotSpecDevice};
use crate::device::wifi_device::ExitCode;
use crate::proto::miio_proto::MiIOProtocol;

// mesh 设备
pub struct MeshDevice {
    pub info: DeviceInfo,
    gateway: Arc<OpenMiioGatewayDevice>,
}

impl MiotSpecDevice for MeshDevice {
    fn get_info(&self) -> &DeviceInfo { &self.info }

    fn get_proto(&self) ->BoxFuture<Result<Arc<MiIOProtocol>, ExitCode>>{
        self.gateway.get_proto()
    }

    fn run(&self) -> BoxFuture<Result<(), ExitCode>> {
        todo!()
    }
}

impl MeshDevice {
    pub fn new(info: DeviceInfo, gateway: Arc<OpenMiioGatewayDevice>,) -> MeshDevice {
        MeshDevice {
            info,
            gateway,
        }
    }
}