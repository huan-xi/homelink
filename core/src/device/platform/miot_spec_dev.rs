use miot_proto::device::gateway::gateway::OpenMiioGatewayDevice;
use miot_proto::device::mesh_device::MeshDevice;
use miot_proto::device::miot_spec_device::AsMiotDevice;
use miot_proto::device::wifi_device::{WifiDevice};
use crate::device::platform::PlatformDevice;


impl PlatformDevice for WifiDevice {}


impl PlatformDevice for OpenMiioGatewayDevice {}

impl <T:AsMiotDevice> PlatformDevice for MeshDevice<T> {}


// impl<T: MiCloudExt > PlatformDevice for MiCloudDevice<T> {}
