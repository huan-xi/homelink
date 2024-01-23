use std::time::Duration;
use futures_util::future::BoxFuture;
use futures_util::{FutureExt, StreamExt};
use crate::device::miot_spec_device::{BaseMiotSpecDevice, DeviceInfo, MiotSpecDevice};
use crate::device::MiotDevicePointer;
use crate::proto::miio_proto::MiotSpecProtocolPointer;
use crate::proto::protocol::ExitError;
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

    fn run(&self) -> BoxFuture<Result<(), ExitError>> {
        async move {
            loop {
                tokio::time::sleep(Duration::from_secs(100)).await;
            }
            Ok(())
        }.boxed()
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