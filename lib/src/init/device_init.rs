use std::sync::Arc;

use anyhow::anyhow;
use log::error;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait};
use sea_orm::QueryFilter;
use tap::TapFallible;

use miot_spec::device::ble::ble_device::BleDevice;
use miot_spec::device::gateway::gateway::OpenMiioGatewayDevice;
use miot_spec::device::miot_spec_device::{AsMiotGatewayDevice, DeviceInfo};
use miot_spec::device::wifi_device::{WifiDevice, WifiDeviceInner};

use crate::db::entity::iot_device::{DeviceParam, IotDeviceType};
use crate::db::entity::prelude::{IotDeviceColumn, IotDeviceEntity, IotDeviceModel, MiotDeviceEntity};
use crate::init::DevicePointer;
use crate::init::manager::device_manager::IotDeviceManager;
use crate::init::manager::mi_account_manager::{MiAccountManager, MiCloudDeviceExt};




