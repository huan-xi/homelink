use sea_orm::ActiveValue::Set;
use crate::db::entity::common::{Property, PropertyVec};
use crate::db::entity::iot_device::SourcePlatform;
use crate::db::entity::prelude::{HapAccessoryActiveModel, HapCharacteristicActiveModel, HapServiceActiveModel, IotDeviceActiveModel, IotDeviceModel, MiotDeviceModel};
use crate::db::SNOWFLAKE;
use crate::hap::template::miot_template::{AccessoryTemplate, CharacteristicTemplate, DeviceTemplate, ServiceTemplate};

#[derive(Clone)]
pub struct AccessoryCtx {
    pub(crate) aid: i64,
    pub(crate) bridge_id: i64,
    pub(crate) dev_ctx: DeviceModelCtx,
}

pub fn to_service_model(aid: i64, sid: i64, svc: &ServiceTemplate) -> anyhow::Result<HapServiceActiveModel> {
    Ok(HapServiceActiveModel {
        id: Set(sid),
        tag: Set(Some(svc.tag.clone())),
        accessory_id: Set(aid),
        disabled: Set(false),
        configured_name: Set(svc.configured_name.clone()),
        service_type: Set(svc.service_type),
        memo: Set(svc.desc.clone()),
        primary: Set(svc.primary),
    })
}

pub fn to_char_model(sid: i64, char: &CharacteristicTemplate) -> anyhow::Result<HapCharacteristicActiveModel> {
    Ok(HapCharacteristicActiveModel {
        cid: Set(SNOWFLAKE.next_id()),
        service_id: Set(sid),
        disabled: Set(false),
        name: Set(char.name.clone()),
        characteristic_type: Set(char.char_type),
        mapping_method: Set(char.mapping_method),
        mapping_param: Set(char.mapping_param.clone()),
        unit_convertor: Set(char.unit_convertor.clone()),
        convertor_param: Set(char.convertor_param.clone()),
        info: Set(char.info.clone()),
    })
}

pub fn to_accessory_model(ctx: AccessoryCtx, accessory: &AccessoryTemplate) -> anyhow::Result<HapAccessoryActiveModel> {
    Ok(HapAccessoryActiveModel {
        aid: Set(ctx.aid),
        name: Set(accessory.name.clone().unwrap_or(ctx.dev_ctx.name.clone())),
        tag: Set(Some(accessory.tag.clone())),
        device_id: Set(ctx.dev_ctx.device_id),
        bridge_id: Set(ctx.bridge_id),
        disabled: Set(false),
        category: Set(accessory.category),
        script: Default::default(),
        model: Set(accessory.model.clone()),
        model_params: Default::default(),
        memo: Set(accessory.desc.clone()),
        info: Default::default(),
        temp_id: Set(Some(ctx.dev_ctx.id.clone())),
        create_at: Set(chrono::Local::now().naive_local()),
        update_at: Set(chrono::Local::now().naive_local()),
    })
}


#[derive(Clone)]
pub struct DeviceModelCtx {
    pub(crate) device_id: i64,
    pub(crate) name: String,
    pub(crate) id: String,
    pub(crate) version: String,
    pub did: String,
}

pub fn to_device_model(ctx: DeviceModelCtx, device: &DeviceTemplate) -> anyhow::Result<IotDeviceActiveModel> {
    Ok(IotDeviceActiveModel {
        device_id: Set(ctx.device_id),
        tag: Set(Some(device.tag.clone())),
        device_type: Set(device.device_type),
        params: Set(device.params.clone()),
        gateway_id: Default::default(),
        name: Set(ctx.name),
        memo: Set(device.desc.clone()),
        disabled: Set(false),
        source_type: Set(Some(SourcePlatform::MiHome)),
        source_id: Set(Some(ctx.did.clone())),
        poll_interval: Set(device.interval.unwrap_or(60_000) as i32),
        timeout: Set(device.timeout.unwrap_or(1_000) as i32),
        poll_properties: Set(PropertyVec(device.poll_properties
            .iter()
            .map(|i| Property::new(i.siid, i.piid))
            .collect::<Vec<Property>>())
        ),
        props: Default::default(),
        services: Default::default(),
        events: Default::default(),
        temp_id: Set(Some(ctx.id)),
        temp_version: Set(Some(ctx.version)),
        update_at: Set(chrono::Utc::now()),
    })
}

