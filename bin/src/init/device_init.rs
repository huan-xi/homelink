use std::collections::HashMap;
use std::sync::Arc;
use anyhow::anyhow;
use log::{error, info};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait};
use miot_spec::device::ble::ble_device::BleDevice;
use miot_spec::device::gateway::gateway::OpenMiioGatewayDevice;
use miot_spec::device::wifi_device::WifiDevice;
use crate::db::entity::iot_device::{DeviceParam, IotDeviceType, Model};
use crate::db::entity::prelude::{IotDeviceEntity, IotDeviceColumn, IotDeviceModel, MiotDeviceEntity};
use crate::init::{DeviceMap, DevicePointer};
use sea_orm::QueryFilter;
use tap::TapFallible;
use miot_spec::device::mesh_device::MeshDevice;
use miot_spec::device::miot_spec_device::DeviceInfo;
use miot_spec::device::MiotDevicePointer;
use crate::init::manager::device_manager::IotDeviceManager;

/// 初始化hap 设备 init_iot_device
pub async fn init_iot_devices(conn: &DatabaseConnection, manage: IotDeviceManager) -> anyhow::Result<()> {
    // let mut map: DeviceMap = dashmap::DashMap::new();
    let mut list = vec![];
    //读取设备
    let devices: Vec<IotDeviceModel> = IotDeviceEntity::find()
        .filter(IotDeviceColumn::GatewayId.is_null().and(
            IotDeviceColumn::Disabled.eq(false)
        ))
        .all(conn)
        .await?;
    for device in devices.into_iter() {
        let device_id = device.device_id;
        if device.source_type.is_none() {
            info!("设备{}没有来源类型", device_id);
            continue;
        };
        match init_mi_device(conn, device).await {
            Ok(dev) => {
                list.push((device_id, dev));
            }
            Err(err) => {
                error!("初始化设备失败，{}", err);
            }
        }
    }
    //初始化米家网关子设备
    let children_devices = IotDeviceEntity::find()
        .filter(IotDeviceColumn::GatewayId.is_not_null().and(
            IotDeviceColumn::Disabled.eq(false)
        ))
        .all(conn)
        .await?;
    for dev in children_devices.into_iter() {
        let dev_id = dev.device_id;
        let parent = list.iter().find(|(id, _)| {
            *id == dev.gateway_id.unwrap_or(-1)
        });

        let res = match parent {
            None => {
                Err(anyhow!(("初始化设备失败，网关不存在")))
            }
            Some((i, gw)) => {
                init_children_device(dev, gw.device.clone()).await
            }
        };
        match res {
            Ok(dev) => {
                list.push((dev_id, dev));
            }
            Err(err) => {
                error!("初始化设备失败，{}", err);
            }
        }
    }

    list.into_iter()
        .for_each(|(id, dev)| {
            manage.push_device(id, dev);
        });
    Ok(())
}


/// 初始化子设备
async fn init_children_device(dev: IotDeviceModel, gw: MiotDevicePointer) -> anyhow::Result<DevicePointer> {
    return match dev.device_type {
        IotDeviceType::MiBleDevice => {
            if let Some(DeviceParam::BleParam(param)) = dev.params {
                // param.mapping
                let mapping = param.get_mapping();
                let ble_dev = BleDevice::new(param.info, gw.clone(), mapping);
                return Ok(DevicePointer::new(Arc::new(ble_dev)));
            }
            Err(anyhow!("初始化设备失败，参数类型错误"))
        }
        IotDeviceType::MiMeshDevice => {
            if let Some(DeviceParam::MeshParam(param)) = dev.params {
                let ble_dev = MeshDevice::new(param, gw.clone());
                return Ok(DevicePointer::new(Arc::new(ble_dev)));
            }
            Err(anyhow!("初始化设备失败，参数类型错误"))
        }
        IotDeviceType::MiCloudDevice => {
            ///配置米家账号

            todo!();
            /*if let Some(DeviceParam::MeshParam(param)) = dev.params {
                let ble_dev = MeshDevice::new(param, gw.clone());
                return Ok(DevicePointer::new(Arc::new(ble_dev)));
            }
            Err(anyhow!("初始化设备失败，参数类型错误"))*/
        }
        _ => {
            Err(anyhow!("初始化设备失败，设备类型错误"))
        }
    };
}

/// 初始化米家不需要网关的设备
async fn init_mi_device(conn: &DatabaseConnection, dev: IotDeviceModel) -> anyhow::Result<DevicePointer> {
    let source_id = dev.source_id.ok_or(anyhow!("设备来源id不存在"))?;
    let param = get_device_info(conn,source_id.as_str()).await?;
    return match dev.device_type {
        IotDeviceType::MiWifiDevice => {
            if let Some(DeviceParam::WifiDeviceParam) = dev.params {
                return Ok(DevicePointer::new(Arc::new(WifiDevice::new(param).await?)));
            }
            Err(anyhow!("初始化设备失败，参数类型错误"))
        }
        IotDeviceType::MiGatewayDevice => {
            if let Some(DeviceParam::MiGatewayParam) = dev.params {
                let dev = OpenMiioGatewayDevice::new(param).await?;
                let dev = Arc::new(dev);
                return Ok(DevicePointer::new(dev.clone()));
            }
            Err(anyhow!("初始化设备失败，参数类型错误"))
        }
        _ => {
            Err(anyhow!("初始化设备失败，设备类型错误"))
        }
    };
}

async fn get_device_info(conn: &DatabaseConnection, id: &str) -> anyhow::Result<DeviceInfo> {
    let miot = MiotDeviceEntity::find_by_id(id.to_string()).one(conn)
        .await?
        .ok_or(anyhow!("米家设备did:{}不存在",id))?;
    let info: DeviceInfo = serde_json::from_value(miot.full)
        .tap_err(|e| error!("米家设备字段不全:{}",e))?;
    Ok(info)
}