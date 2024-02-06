use miot_spec::device::gateway::gateway::OpenMiioGatewayDevice;
use miot_spec::device::wifi_device::{WifiDevice};
use crate::device::platform::PlatformDevice;


impl PlatformDevice for WifiDevice {}


impl PlatformDevice for OpenMiioGatewayDevice {}


// impl<T: MiCloudExt > PlatformDevice for MiCloudDevice<T> {}
