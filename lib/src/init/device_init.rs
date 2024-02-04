use std::sync::Arc;

use anyhow::anyhow;
use log::{error, info};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait};
use sea_orm::QueryFilter;
use tap::TapFallible;

use miot_spec::device::ble::ble_device::BleDevice;
use miot_spec::device::cloud_device::MiCloudDevice;
use miot_spec::device::gateway::gateway::OpenMiioGatewayDevice;
use miot_spec::device::mesh_device::MeshDevice;
use miot_spec::device::miot_spec_device::DeviceInfo;
use miot_spec::device::MiotDevicePointer;
use miot_spec::device::wifi_device::WifiDevice;

use crate::db::entity::iot_device::{DeviceParam, IotDeviceType};
use crate::db::entity::prelude::{IotDeviceColumn, IotDeviceEntity, IotDeviceModel, MiotDeviceEntity};
use crate::init::DevicePointer;
use crate::init::manager::device_manager::IotDeviceManager;
use crate::init::manager::mi_account_manager::{MiAccountManager, MiCloudDeviceExt};

/// 初始化hap 设备 init_iot_device
pub async fn init_iot_device_manager(conn: &DatabaseConnection, manage: IotDeviceManager, mi_account_manager: MiAccountManager) -> anyhow::Result<()> {
    // let mut map: DeviceMap = dashmap::DashMap::new();
    // let mut list = vec![];
    //读取设备
    let devices: Vec<IotDeviceModel> = IotDeviceEntity::find()
        .filter(IotDeviceColumn::GatewayId.is_null().and(
            IotDeviceColumn::Disabled.eq(false)
        ))
        .all(conn)
        .await?;
    for device in devices.into_iter() {
        if let Err(err) = init_mi_device(conn, device, manage.clone(), mi_account_manager.clone()).await {
            error!("初始化设备失败，{}", err);
        };
    }
    //初始化米家网关子设备
    let children_devices = IotDeviceEntity::find()
        .filter(IotDeviceColumn::GatewayId.is_not_null().and(
            IotDeviceColumn::Disabled.eq(false)
        ))
        .all(conn)
        .await?;
    for dev in children_devices.into_iter() {
        if let Err(err) = init_children_device(conn, dev, manage.clone()).await {
            error!("初始化子设备失败，{}", err);
        }
    }
    Ok(())
}


/// 初始化子设备
pub async fn init_children_device(conn: &DatabaseConnection, dev: IotDeviceModel, manager: IotDeviceManager) -> anyhow::Result<()> {
    let dev_id = dev.device_id;

    let gw_id = match dev.gateway_id {
        None => {
            return Err(anyhow!("设备{}未设置网关id", dev_id));
        }
        Some(s) => s
    };

    let gw = match manager.get_device(gw_id) {
        None => {
            return Err(anyhow!("初始化设备失败，网关:{}不存在,或未启动",gw_id));
        }
        Some(gw) => gw
    };
    let dev = init_children_device0(conn, dev, gw.device).await?;
    manager.push_device(dev_id, dev);
    Ok(())
}


async fn init_children_device0(conn: &DatabaseConnection, dev: IotDeviceModel, gw: MiotDevicePointer) -> anyhow::Result<DevicePointer> {
    //查米家设备
    let source_id = dev.source_id.ok_or(anyhow!("设备来源id不存在"))?;
    let (_, dev_info) = get_device_info(conn, source_id.as_str()).await?;
    return match dev.device_type {
        IotDeviceType::MiBleDevice => {
            if let Some(DeviceParam::BleParam(param)) = dev.params {
                // param.mapping
                let mapping = param.get_mapping();
                let ble_dev = BleDevice::new(param.info, gw.clone(), mapping);
                return Ok(DevicePointer::new(Arc::new(ble_dev)));
            }
            Err(anyhow!("初始化设备失败，MiBleDevice 参数类型错误"))
        }
        IotDeviceType::MiMeshDevice => {
            if let Some(DeviceParam::MeshParam) = dev.params {
                let ble_dev = MeshDevice::new(dev_info, gw.clone());
                return Ok(DevicePointer::new(Arc::new(ble_dev)));
            }
            Err(anyhow!("初始化设备失败，MiMeshDevice参数类型错误"))
        }
        _ => {
            Err(anyhow!("初始化设备失败，设备类型错误"))
        }
    };
}

/// 初始化米家不需要网关的设备
pub async fn init_mi_device(conn: &DatabaseConnection, dev: IotDeviceModel, manager: IotDeviceManager, mi_account_manager: MiAccountManager) -> anyhow::Result<()> {
    let device_id = dev.device_id;
    if dev.source_type.is_none() {
        return Err(anyhow!("设备{}没有来源类型", device_id));
    };
    let dev = init_mi_device0(conn, dev, &mi_account_manager).await?;
    manager.push_device(device_id, dev);
    Ok(())
}

async fn init_mi_device0(conn: &DatabaseConnection, dev: IotDeviceModel, mi_account_manager: &MiAccountManager) -> anyhow::Result<DevicePointer> {
    let source_id = dev.source_id.ok_or(anyhow!("设备来源id不存在"))?;
    let (account_id, param) = get_device_info(conn, source_id.as_str()).await?;
    return match dev.device_type {
        IotDeviceType::MiWifiDevice => {
            return Ok(DevicePointer::new(Arc::new(WifiDevice::new(param).await?)));
        }
        IotDeviceType::MiGatewayDevice => {
            let dev = OpenMiioGatewayDevice::new(param).await?;
            let dev = Arc::new(dev);
            return Ok(DevicePointer::new(dev.clone()));
        }
        IotDeviceType::MiCloudDevice => {
            // mi_account_manager.get_proto(account_id.as_str()).await?;
            //todo 判断米家账号状态
            let ext = MiCloudDeviceExt::new(account_id, mi_account_manager.clone());
            Ok(DevicePointer::new(Arc::new(MiCloudDevice::new(param, ext))))
        }
        _ => {
            Err(anyhow!("初始化设备失败，设备类型错误"))
        }
    };
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