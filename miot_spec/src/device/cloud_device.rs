use std::sync::Arc;
use std::time::Duration;
use futures_util::future::BoxFuture;
use impl_new::New;
use crate::device::miot_spec_device::{BaseMiotSpecDevice, DeviceInfo, MiotSpecDevice};
use crate::proto::miio_proto::MiotSpecProtocolPointer;
use crate::proto::protocol::ExitError;
use crate::proto::transport::cloud_miio_proto::CloudMiioProto;

/// 通过云端接入的设备
pub struct MiCloudDevice {
    pub base: BaseMiotSpecDevice,
    pub info: DeviceInfo,
    ///协议
    proto: Arc<CloudMiioProto>,
}

impl MiCloudDevice {
    pub fn new(info: DeviceInfo, proto: Arc<CloudMiioProto>) -> Self {
        Self {
            base: Default::default(),
            info,
            proto,
        }
    }
}

#[async_trait::async_trait]
impl MiotSpecDevice for MiCloudDevice {
    fn get_info(&self) -> &DeviceInfo {
        &self.info
    }

    fn get_base(&self) -> &BaseMiotSpecDevice {
        &self.base
    }

    async fn get_proto(&self) -> Result<MiotSpecProtocolPointer, ExitError> {
        Ok(self.proto.clone())
    }

    fn run(&self) -> BoxFuture<Result<(), ExitError>> {
        todo!()
    }
}