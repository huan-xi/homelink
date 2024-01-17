use std::collections::HashMap;
use std::sync::Arc;
use anyhow::anyhow;
use log::error;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait};
use miot_spec::device::ble::ble_device::BleDevice;
use miot_spec::device::gateway::gateway::OpenMiioGatewayDevice;
use miot_spec::device::wifi_device::WifiDevice;
use crate::db::entity::iot_device::{DeviceParam, IotDeviceType, Model};
use crate::db::entity::prelude::{IotDevice, IotDeviceColumn, IotDeviceModel};
use crate::init::{DeviceMap, DevicePointer};
use sea_orm::QueryFilter;
use miot_spec::device::mesh_device::MeshDevice;
use crate::init::device_manage::IotDeviceManager;

/// 初始化hap 设备 init_iot_device
pub async fn init_iot_devices(conn: &DatabaseConnection,manage:IotDeviceManager) -> anyhow::Result<()> {
    // let mut map: DeviceMap = dashmap::DashMap::new();
    let mut list = vec![];
    //读取设备
    let devices: Vec<IotDeviceModel> = IotDevice::find()
        .filter(IotDeviceColumn::GatewayId.is_null().and(
            IotDeviceColumn::Disabled.eq(false)
        ))
        .all(conn)
        .await?;
    let mut mi_gateway_map = dashmap::DashMap::new();
    for device in devices.into_iter() {
        let device_id = device.device_id;
        match init_mi_device(device).await {
            Ok((dev, gate)) => {
                list.push((device_id, dev));
                if let Some(g) = gate {
                    mi_gateway_map.insert(device_id, g);
                };
            }
            Err(err) => {
                error!("初始化设备失败，{}", err);
            }
        }
    }
    //初始化米家网关子设备
    let children_devices = IotDevice::find()
        .filter(IotDeviceColumn::GatewayId.is_not_null().and(
            IotDeviceColumn::Disabled.eq(false)
        ))
        .all(conn)
        .await?;
    for dev in children_devices.into_iter() {
        let dev_id = dev.device_id;
        let res = match mi_gateway_map.get(&dev.gateway_id.unwrap_or(-1)) {
            None => {
                Err(anyhow!(("初始化设备失败，网关不存在")))
            }
            Some(gw) => {
                init_children_device(dev, gw.clone()).await
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

async fn init_children_device(dev: IotDeviceModel, gw: Arc<OpenMiioGatewayDevice>) -> anyhow::Result<DevicePointer> {
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
        _ => {
            Err(anyhow!("初始化设备失败，设备类型错误"))
        }
    };
}

/// 初始化米家不需要网关的设备
async fn init_mi_device(dev: IotDeviceModel) -> anyhow::Result<(DevicePointer, Option<Arc<OpenMiioGatewayDevice>>)> {
    return match dev.device_type {
        IotDeviceType::MiWifiDevice => {
            if let Some(DeviceParam::WifiDeviceParam(param)) = dev.params {
                return Ok((DevicePointer::new(Arc::new(WifiDevice::new(param).await?)), None));
            }
            Err(anyhow!("初始化设备失败，参数类型错误"))
        }
        IotDeviceType::MiGatewayDevice => {
            if let Some(DeviceParam::MiGatewayParam(param)) = dev.params {
                let dev = OpenMiioGatewayDevice::new(param).await?;
                let dev = Arc::new(dev);
                return Ok((DevicePointer::new(dev.clone()), Some(dev)));
            }
            Err(anyhow!("初始化设备失败，参数类型错误"))
        }
        _ => {
            Err(anyhow!("初始化设备失败，设备类型错误"))
        }
    };
}