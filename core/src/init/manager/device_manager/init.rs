use std::str::FromStr;
use std::sync::Arc;

use anyhow::anyhow;
use log::error;
use sea_orm::*;

use miot_proto::device::miot_spec_device::{AsMiotDevice, MiotDeviceArc, MiotDeviceType};

use crate::db::entity::iot_device::SourcePlatform;
use crate::db::entity::prelude::{IotDeviceColumn, IotDeviceEntity, IotDeviceModel};
use crate::init::manager::device_manager::IotDeviceManagerInner;

impl IotDeviceManagerInner {
    pub async fn init(&self) -> anyhow::Result<()> {
        let devices: Vec<IotDeviceModel> = IotDeviceEntity::find()
            .filter(IotDeviceColumn::GatewayId.is_null().and(
                IotDeviceColumn::Disabled.eq(false)
            ))
            .all(&self.conn)
            .await?;
        // 先处理不需要网关的设备
        for device in devices.into_iter() {
            if let Err(err) = self.init_no_gw(device).await {
                error!("初始化设备失败，{}", err);
            };
        }

        //继续初始化,网关子设备
        let children_devices = IotDeviceEntity::find()
            .filter(IotDeviceColumn::GatewayId.is_not_null().and(
                IotDeviceColumn::Disabled.eq(false)
            ))
            .all(&self.conn)
            .await?;
        for dev in children_devices.into_iter() {
            if let Err(err) = self.init_children_device(dev).await {
                error!("初始化子设备失败，{}", err);
            }
        }


        Ok(())
    }
    /// 不需要网关的设备
    pub async fn init_no_gw(&self, dev: IotDeviceModel) -> anyhow::Result<()> {
        let dev_id = dev.device_id;
        let dev = match dev.source_platform {
            SourcePlatform::MiHome => {
                self.init_mi_device_no_gw(dev).await?
            }
            SourcePlatform::NativeBle => {
                self.init_native_ble(dev).await?
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
        let dev = match dev.source_platform {
            SourcePlatform::MiHome => {
                self.init_mi_device_child(dev, MiotDeviceArc(gw)).await?
            }
            _ => {
                return Err(anyhow!("暂不支持本地蓝牙设备网关接入"));
            }
        };
        self.push_device(dev_id, dev);
        Ok(())
    }

    pub fn test(&self, arc: Arc<dyn AsMiotDevice>) {

    }
}


