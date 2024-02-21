use std::str::FromStr;
use std::sync::Arc;
use anyhow::anyhow;
use log::error;
use sea_orm::DatabaseConnection;
use tap::TapFallible;
use miot_proto::device::miot_spec_device::{AsMiotDevice, DeviceInfo, MiotDeviceType};
use crate::db::entity::prelude::{IotDeviceModel, MiotDeviceEntity};
use sea_orm::*;
use miot_proto::device::ble::ble_device::BleDevice;
use miot_proto::device::cloud_device::{MiCloudDevice, MiCloudDeviceInner};
use miot_proto::device::gateway::gateway::OpenMiioGatewayDevice;
use miot_proto::device::mesh_device::MeshDevice;
use miot_proto::device::wifi_device::WifiDevice;
use crate::db::entity::iot_device::{DeviceParam, IotDeviceType};
use crate::init::DevicePointer;
use crate::init::manager::device_manager::IotDeviceManagerInner;
use crate::init::manager::mi_account_manager::MiCloudDeviceExt;


impl IotDeviceManagerInner {
    /// 初始化米家子设备
    pub(crate) async fn init_mi_device_child<T: AsMiotDevice + 'static>(&self, dev: IotDeviceModel, gw: T) -> anyhow::Result<DevicePointer> {
        //查米家设备
        let source_id = dev.source_id.ok_or(anyhow!("设备来源id不存在"))?;
        let (_, dev_info) = get_device_info(&self.conn, source_id.as_str()).await?;
        let device_type = MiotDeviceType::from_str(dev.device_type.as_str())?;
        return match device_type {
            MiotDeviceType::Ble => {
                let ble_dev = BleDevice::new(dev_info, gw);
                return Ok(Arc::new(ble_dev));
            }
            MiotDeviceType::Mesh => {
                let mesh_dev = MeshDevice::new_mesh_device(dev_info, gw);
                return Ok(Arc::new(mesh_dev));
            }
            _ => {
                Err(anyhow!("初始化设备失败，设备类型错误,{:?}设备不支持子设备",device_type))
            }
        };
    }

    /// 初始化米家不需要网关的设备
    pub(crate) async fn init_mi_device_no_gw(&self, dev: IotDeviceModel) -> anyhow::Result<DevicePointer> {
        let source_id = dev.source_id.ok_or(anyhow!("米家设备来源id不存在"))?;
        let (account_id, param) = get_device_info(&self.conn, source_id.as_str()).await?;
        let device_type = MiotDeviceType::from_str(dev.device_type.as_str())?;
        return match device_type {
            MiotDeviceType::Wifi => {
                return Ok(Arc::new(WifiDevice::new_wifi_device(param, dev.params)?));
            }
            MiotDeviceType::MqttGateway => {
                let dev = OpenMiioGatewayDevice::new_open_gateway(param)?;
                let dev = Arc::new(dev);
                return Ok(dev.clone());
            }
            MiotDeviceType::Cloud => {
                // mi_account_manager.get_proto(account_id.as_str()).await?;
                let ext = MiCloudDeviceExt::new(account_id, self.mi_account_manager.clone());
                let dev = MiCloudDeviceInner::new(param, ext);
                let dev = MiCloudDevice::new_cloud_device(Box::new(dev));
                return Ok(Arc::new(dev));
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