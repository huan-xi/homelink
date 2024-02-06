mod miot_spec_dev;
mod hap;

use serde_json::Value;
use hl_device::event::HlDeviceListenable;
use hl_device::HlDevice;
use hl_device::platform::hap::hap_device_ext::AsHapDeviceExt;
use miot_spec::device::miot_spec_device::{AsMiotDevice, AsMiotGatewayDevice};
use miot_spec::proto::miio_proto::MiotSpecId;

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
}
