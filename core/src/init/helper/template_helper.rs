use anyhow::anyhow;
use sea_orm::ActiveValue::Set;
use target_hap::types::{CharIdentifier, HapCharInfo, ModelDelegateParam};
use crate::db::entity::hap_accessory::ModelDelegateParamVec;
use crate::db::entity::hap_characteristic::HapCharInfoQueryResult;
use crate::db::entity::iot_device::SourcePlatform;
use crate::db::entity::prelude::{HapAccessoryActiveModel, HapCharacteristicActiveModel, HapServiceActiveModel, IotDeviceActiveModel, IotDeviceModel, MiotDeviceModel};
use crate::db::SNOWFLAKE;
use crate::template::miot_template::{AccessoryTemplate, CharacteristicTemplate,  DeviceTemplate, ServiceTemplate};

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
        service_type: Set(svc.service_type.to_string()),
        memo: Set(svc.memo.clone()),
        primary: Set(svc.primary),
    })
}

pub fn to_char_model(sid: i64, char: &CharacteristicTemplate, default: HapCharInfo) -> anyhow::Result<HapCharacteristicActiveModel> {
    let info_temp = char.info.clone();
    let info = HapCharInfo {
        format: info_temp.format.unwrap_or(default.format),
        unit: info_temp.unit.or(default.unit),
        min_value: info_temp.min_value.or(default.min_value),
        max_value: info_temp.max_value.or(default.max_value),
        step_value: info_temp.step_value.or(default.step_value),
        max_len: info_temp.max_len.or(default.max_len),
        max_data_len: info_temp.max_data_len.or(default.max_data_len),
        valid_values: info_temp.valid_values.or(default.valid_values),
        valid_values_range: info_temp.valid_values_range.or(default.valid_values_range),
        ttl: info_temp.ttl.or(default.ttl),
        perms: info_temp.perms.unwrap_or(default.perms),
        pid: info_temp.pid.or(default.pid),
    };

    Ok(HapCharacteristicActiveModel {
        cid: Set(SNOWFLAKE.next_id()),
        service_id: Set(sid),
        disabled: Set(false),
        name: Set(char.name.clone()),
        characteristic_type: Set(char.char_type.to_string()),
        // mapping_method: Set(char.mapping_method),
        // mapping_param: Set(char.mapping_param.clone()),
        convertor: Set(char.convertor.clone()),
        convertor_param: Set(char.convertor_param.clone()),
        memo: Set(char.memo.clone()),
        info: Set(HapCharInfoQueryResult(info)),
        ..Default::default()
    })
}

pub fn to_accessory_model(ctx: AccessoryCtx, accessory: &AccessoryTemplate) -> anyhow::Result<HapAccessoryActiveModel> {
    //Vec<ModelDelegateParam>
    let mut delegates = accessory.hap_delegates.clone();
    if let Some(s)=accessory.hap_delegate.clone() {
        delegates.push(s);
    }
    if delegates.len() > 1 && delegates.get(0).as_ref().unwrap().chars.is_none() {
        return Err(anyhow!("hap_delegates chars is none"));
    }

    let hap_model_delegates = delegates
        .iter()
        .map(|i| {
            let chars = i.chars
                .clone()
                .unwrap_or_else(|| {
                    let mut chars = vec![];
                    for s in accessory.services.iter() {
                        for c in s.chars.iter() {
                            chars.push(CharIdentifier::new(s.tag.clone(), c.char_type));
                        }
                    }
                    chars
                });

            ModelDelegateParam {
                chars,
                model: i.model.clone(),
                params: i.params.clone(),
            }
        }).collect::<Vec<ModelDelegateParam>>();


    Ok(HapAccessoryActiveModel {
        aid: Set(ctx.aid),
        name: Set(accessory.name.clone().unwrap_or(ctx.dev_ctx.name.clone())),
        tag: Set(Some(accessory.tag.clone())),
        device_id: Set(ctx.dev_ctx.device_id),
        bridge_id: Set(ctx.bridge_id),
        disabled: Set(false),
        category: Set(accessory.category),
        hap_model_delegates: Set(ModelDelegateParamVec(hap_model_delegates)),
        // script: Default::default(),
        // script_params: Default::default(),
        // model: Set(accessory.model.clone()),
        // model_params: Set(accessory.model_params.clone()),
        memo: Set(accessory.memo.clone()),
        info: Default::default(),
        temp_id: Set(Some(ctx.dev_ctx.id.clone())),
        create_at: Set(chrono::Local::now().naive_local()),
        update_at: Set(chrono::Local::now().naive_local()),
        ..Default::default()
    })
}


#[derive(Clone)]
pub struct DeviceModelCtx {
    pub(crate) device_id: i64,
    pub(crate) temp_batch_id: i64,
    pub(crate) name: String,
    pub(crate) id: String,
    pub(crate) version: String,
    pub did: String,
}

pub fn to_device_model(ctx: DeviceModelCtx, device: &DeviceTemplate) -> anyhow::Result<IotDeviceActiveModel> {
    Ok(IotDeviceActiveModel {
        device_id: Set(ctx.device_id),
        tag: Set(Some(device.tag.clone())),
        device_type: Set(device.device_type.clone()),
        params: Set(device.params.clone()),
        gateway_id: Default::default(),
        name: Set(ctx.name),
        memo: Set(device.memo.clone()),
        disabled: Set(false),
        source_platform: Set(SourcePlatform::Mijia.as_ref().to_string()),
        source_id: Set(Some(ctx.did.clone())),
        temp_id: Set(Some(ctx.id)),
        temp_version: Set(Some(ctx.version)),
        temp_batch_id: Set(Some(ctx.temp_batch_id)),
        update_at: Set(chrono::Utc::now()),
    })
}

