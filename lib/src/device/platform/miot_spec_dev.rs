use miot_spec::device::gateway::gateway::OpenMiioGatewayDevice;
use miot_spec::device::mesh_device::MeshDevice;
use miot_spec::device::miot_spec_device::AsMiotDevice;
use miot_spec::device::wifi_device::{WifiDevice};
use crate::device::platform::PlatformDevice;


impl PlatformDevice for WifiDevice {}


impl PlatformDevice for OpenMiioGatewayDevice {}

impl <T:AsMiotDevice> PlatformDevice for MeshDevice<T> {}


// impl<T: MiCloudExt > PlatformDevice for MiCloudDevice<T> {}
