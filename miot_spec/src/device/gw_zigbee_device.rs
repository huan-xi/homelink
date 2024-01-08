use std::sync::Arc;
use futures_util::future::BoxFuture;
use crate::device::gateway::gateway::OpenMiioGatewayDevice;
use crate::device::miot_spec_device::{BaseMiotSpecDevice, DeviceInfo, MiotSpecDevice};
use crate::device::value::DataListener;
use crate::device::wifi_device::ExitCode;
use crate::proto::miio_proto::MiIOProtocol;

/// 通过米家网关的zigbee子设备

pub struct ZigbeeDevice {
    pub info: DeviceInfo,
    gateway: Arc<OpenMiioGatewayDevice>,
}
impl MiotSpecDevice for ZigbeeDevice {
    fn get_info(&self) -> &DeviceInfo { &self.info }

    fn get_proto(&self) -> BoxFuture<Result<Arc<MiIOProtocol>, ExitCode>> {
        self.gateway.get_proto() }

    fn run(&self) -> BoxFuture<Result<(), ExitCode>> {
        todo!()
    }
}
impl ZigbeeDevice {
    pub fn new(info: DeviceInfo, gateway: Arc<OpenMiioGatewayDevice>) -> ZigbeeDevice {
        ZigbeeDevice {
            info,
            gateway,
        }
    }
}


/*impl BleDevice {
    pub fn new(info: DeviceInfo, gateway: Arc<OpenMiioGateway>) -> Self {
        //网关上设置一个监听器,监听属于我的消息

        Self {
            info,
            base: BaseMiotSpecDevice {
                ..BaseMiotSpecDevice::default()
            },
            gateway,
            values: Default::default(),
            emitter: Arc::new(Default::default()),
        }
    }
    pub async fn emit(&self, event: BleValue) {
        debug!("emitting event: {:?}", event);
        self.emitter.read().await.emit(event).await;
    }
    pub fn add_listener(&self, listener: DataListener<BleValue>) -> BoxFuture<()> {
        async move {
            self.emitter.clone().write().await.add_listener(listener).await;
        }.boxed()
    }
}
*/