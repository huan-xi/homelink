use std::str::FromStr;
use std::sync::Arc;

use anyhow::anyhow;
use log::error;
use sea_orm::*;

use miot_proto::device::miot_spec_device::{AsMiotDevice, MiotDeviceArc};
use crate::db::entity::iot_device::DeviceType;
use crate::db::entity::prelude::{IotDeviceColumn, IotDeviceEntity, IotDeviceModel};
use crate::init::manager::device_manager::IotDeviceManagerInner;

impl IotDeviceManagerInner {
    pub async fn init(&self) -> anyhow::Result<()> {
        self.init_devices(None).await?;
        Ok(())
    }

    pub async fn init_devices(&self, filter_ids: Option<Vec<i64>>) -> anyhow::Result<()> {
        let mut condition = Condition::all();
        condition = condition
            .add(IotDeviceColumn::Disabled.eq(false))
            .add(IotDeviceColumn::DeviceType.ne(DeviceType::Child));
        if let Some(ids) = filter_ids.as_ref() {
            condition = condition.add(IotDeviceColumn::DeviceId.is_in(ids.clone()));
        };

        let devices: Vec<IotDeviceModel> = IotDeviceEntity::find()
            .filter(condition)
            .all(&self.conn)
            .await?;
        // 先处理不需要网关的设备
        for device in devices.into_iter() {
            let name = device.name.clone();
            if let Err(err) = self.init_no_gw(device).await {
                error!("初始化设备:{name}失败，{}", err);
            };
        }

        //继续初始化,网关子设备
        let mut children_filter = IotDeviceColumn::DeviceType.eq(DeviceType::Child)
            .and(
                IotDeviceColumn::Disabled.eq(false)
            );
        if let Some(ids) = filter_ids.clone() {
            children_filter = children_filter.and(IotDeviceColumn::DeviceId.is_in(ids));
        };

        let children_devices = IotDeviceEntity::find()
            .filter(children_filter)
            .all(&self.conn)
            .await?;
        for dev in children_devices.into_iter() {
            let name = dev.name.clone();
            if let Err(err) = self.init_children_device(dev).await {
                error!("初始化子设备:{name}失败，{}", err);
            }
        }


        Ok(())
    }
    /// 不需要网关的设备
    pub async fn init_no_gw(&self, dev: IotDeviceModel) -> anyhow::Result<()> {
        let dev_id = dev.device_id;
        let dev = match dev.source_platform.as_str() {
            "mijia" => {
                self.init_mi_device_no_gw(dev).await?
            }
            "ble-native" => {
                self.init_native_ble(dev).await?
            }
            _ => {
                return Err(anyhow!("暂不支持:{}类型设备接入",dev.source_platform.as_str()));
            }
        };
        self.push_device(dev_id, dev);
        Ok(())
    }

    pub async fn init_children_device(&self, dev: IotDeviceModel) -> anyhow::Result<()> {
        let dev_id = dev.device_id;
        let gw_id = dev.gateway_id.ok_or(anyhow!("设备{}未设置网关id", dev_id))?;
        let gw = self.get_device(gw_id)
            .ok_or(anyhow!("网关设备{}不存在", gw_id))?;
        let platform = dev.source_platform.clone();
        let dev = match dev.source_platform.as_str() {
            "mijia" => {
                self.init_mi_device_child(dev, MiotDeviceArc(gw)).await?
            }
            _ => {
                return Err(anyhow!("暂不支持:{platform}设备网关接入"));
            }
        };
        self.push_device(dev_id, dev);
        Ok(())
    }

    pub fn test(&self, arc: Arc<dyn AsMiotDevice>) {}
}


