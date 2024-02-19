mod miot_spec_dev;
mod hap;

use anyhow::anyhow;
use serde_json::Value;
use hl_integration::event::HlDeviceListenable;
use hl_integration::HlDevice;
use hl_integration::platform::hap::hap_device_ext::AsHapDeviceExt;
use miot_proto::device::miot_proto_device::{AsMiotDevice,  MiotSpecDevice, NotSupportMiotDeviceError};
use miot_proto::proto::miio_proto::MiotSpecId;
use crate::device::native_ble::native_ble_device::NativeBleDevice;

pub struct NotSupportNativeBleDeviceError;

impl From<NotSupportNativeBleDeviceError> for anyhow::Error {
    fn from(value: NotSupportNativeBleDeviceError) -> Self {
        anyhow!("该设备不是蓝牙设备类型")
    }
}


#[async_trait::async_trait]
pub trait PlatformDevice: HlDevice + AsHapDeviceExt + AsMiotDevice + Send + Sync + HlDeviceListenable {
    async fn read_property(&self, siid: i32, piid: i32) -> anyhow::Result<Option<Value>> {
        self.as_miot_device()?
            .read_property(siid, piid).await
    }
    async fn set_property(&self, spec_id: MiotSpecId, value: Value) -> anyhow::Result<()> {
        self.as_miot_device()?
            .set_property(spec_id, value)
            .await
    }
    /* fn as_miot_device(&self) -> Result<&dyn MiotSpecDevice, NotSupportMiotDeviceError> {
         Err(NotSupportMiotDeviceError)
     }*/

    fn as_native_ble(&self) -> Result<&NativeBleDevice, NotSupportNativeBleDeviceError> {
        Err(NotSupportNativeBleDeviceError)
    }
}
