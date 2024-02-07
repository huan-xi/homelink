use std::sync::Arc;
use anyhow::anyhow;
use log::error;
use sea_orm::DatabaseConnection;
use tap::TapFallible;
use miot_spec::device::miot_spec_device::{AsMiotDevice, DeviceInfo, MiotSpecDevice, NotSupportMiotDeviceError};
use crate::db::entity::prelude::{IotDeviceModel, MiotDeviceEntity};
use sea_orm::*;
use miot_spec::device::ble::ble_device::BleDevice;
use miot_spec::device::gateway::gateway::OpenMiioGatewayDevice;
use miot_spec::device::mesh_device::MeshDevice;
use miot_spec::device::wifi_device::WifiDevice;
use crate::db::entity::iot_device::{DeviceParam, IotDeviceType};
use crate::init::DevicePointer;
use crate::init::manager::device_manager::IotDeviceManagerInner;
use crate::init::manager::mi_account_manager::MiCloudDeviceExt;

#[derive(Clone)]
pub struct MiotDeviceArc(pub DevicePointer);

impl AsMiotDevice for MiotDeviceArc {
    fn as_miot_device(&self) -> Result<&dyn MiotSpecDevice, NotSupportMiotDeviceError> {
        self.0.as_miot_device()
    }
}


impl IotDeviceManagerInner {
    /// 初始化米家子设备
    pub(crate) async fn init_mi_device_child<T: AsMiotDevice + 'static>(&self, dev: IotDeviceModel, gw: T) -> anyhow::Result<DevicePointer> {
        //查米家设备
        let source_id = dev.source_id.ok_or(anyhow!("设备来源id不存在"))?;
        let (_, dev_info) = get_device_info(&self.conn, source_id.as_str()).await?;
        return match dev.device_type {
            IotDeviceType::MiBleDevice => {
                if let Some(DeviceParam::BleParam(param)) = dev.params {
                    // param.mapping
                    let mapping = param.get_mapping();
                    let ble_dev = BleDevice::new(param.info, gw, mapping);
                    // return Ok(Arc::new(ble_dev));
                    todo!();
                }
                Err(anyhow!("初始化子设备失败，MiBleDevice 参数类型错误"))
            }
            IotDeviceType::MiMeshDevice => {
                let ble_dev = MeshDevice::new(dev_info, gw);
                return Ok(Arc::new(ble_dev));
            }
            _ => {
                Err(anyhow!("初始化设备失败，设备类型错误,该设备不支持子设备"))
            }
        };
    }

    /// 初始化米家不需要网关的设备
    pub(crate) async fn init_mi_device_no_gw(&self, dev: IotDeviceModel) -> anyhow::Result<DevicePointer> {
        let source_id = dev.source_id.ok_or(anyhow!("米家设备来源id不存在"))?;
        let (account_id, param) = get_device_info(&self.conn, source_id.as_str()).await?;
        return match dev.device_type {
            IotDeviceType::MiWifiDevice => {
                return Ok(Arc::new(WifiDevice::new(param)?));
            }
            IotDeviceType::MiGatewayDevice => {
                let dev = OpenMiioGatewayDevice::new(param).await?;
                let dev = Arc::new(dev);
                return Ok(dev.clone());
            }
            IotDeviceType::MiCloudDevice => {
                // mi_account_manager.get_proto(account_id.as_str()).await?;
                //todo 判断米家账号状态
                let ext = MiCloudDeviceExt::new(account_id, self.mi_account_manager.clone());
                // Ok(Arc::new(MiCloudDevice::new(param, ext)))
                todo!();
            }
            _ => {
                Err(anyhow!("初始化设备失败，设备类型错误,该设备需要网关"))
            }
        };
    }
}


async fn get_device_info(conn: &DatabaseConnection, id: &str) -> anyhow::Result<(String, DeviceInfo)> {
    let miot = MiotDeviceEntity::find_by_id(id.to_string()).one(conn)
        .await?
        .ok_or(anyhow!("米家设备did:{}不存在",id))?;
    let user_id = miot.user_id.clone();

    let info: DeviceInfo = serde_json::from_value(miot.full)
        .tap_err(|e| error!("米家设备字段不全:{}",e))?;

    Ok((user_id, info))
}