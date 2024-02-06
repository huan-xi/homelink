use dashmap::DashMap;
use log::info;
use ble_monitor::BleValue;
use hap::characteristic::{CharReadParam, CharUpdateParam, ReadCharResults, UpdateCharResults};
use hl_device::event::{EventListener, HlDeviceListenable};
use hl_device::hap_platform::hap_device_ext::HapDeviceExt;
use miot_spec::device::miot_spec_device::{AsMiotSpecDevice, BaseMiotSpecDevice, DeviceInfo, MiotSpecDevice};
use miot_spec::proto::miio_proto::MiotSpecProtocolPointer;
use miot_spec::proto::protocol::ExitError;
use crate::init::manager::ble_manager::BleManager;

/// 特征映射map
pub struct NativeBleDevice {
    // fields omitted
    mac: [u8; 6],
    // 存储最近一次的值
    value_register: DashMap<u16, BleValue>,
    ble_manager: BleManager,
}

#[async_trait::async_trait]
impl HlDeviceListenable for NativeBleDevice {
    async fn add_listener(&self, listener: EventListener) {
        todo!()
    }
}

#[async_trait::async_trait]
impl HapDeviceExt for NativeBleDevice{
    async fn read_chars_value(&self, params: Vec<CharReadParam>) -> ReadCharResults {
        todo!()
    }

    async fn update_chars_value(&self, params: Vec<CharUpdateParam>) -> UpdateCharResults {
        todo!()
    }
}

impl AsMiotSpecDevice for NativeBleDevice {
    fn as_miot_spec_device(&self) -> Option<&(dyn MiotSpecDevice + Send + Sync)>{
        None
    }
}

#[async_trait::async_trait]
impl MiotSpecDevice for NativeBleDevice {
    fn get_info(&self) -> &DeviceInfo {
        todo!()
    }

    fn get_base(&self) -> &BaseMiotSpecDevice {
        todo!()
    }

    async fn get_proto(&self) -> Result<MiotSpecProtocolPointer, ExitError> {
        todo!()
    }

    async fn run(&self) -> Result<(), ExitError> {
        ///从蓝牙广播中读数据解码
        let mut recv = self.ble_manager.recv();
        // 读取数据
        while let Ok(data) = recv.recv().await {
            info!("recv data {:?}", data);
            if data.mac==self.mac {
                // 存储数据
                self.value_register.insert(data.etype, data.edata);
            }

        };
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;
    use std::time::Duration;
    use miot_spec::device::miot_spec_device::MiotSpecDevice;
    use crate::init::logger_init::init_logger;

    #[tokio::test]
    pub async fn test() {
        init_logger();
        let ble_manager = super::BleManager::new();
        ble_manager.init().await.unwrap();
        let dev = Arc::new(super::NativeBleDevice {
            mac: [23, 117, 189, 97, 185, 34],
            value_register: Default::default(),
            ble_manager,
        });
        let dev_c = dev.clone();
        tokio::spawn(async move {
            dev_c.run().await.unwrap();
        });

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}