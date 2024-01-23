use deno_runtime::deno_core::url::Url;
use impl_new::New;
use sea_orm::JsonValue;
use serde::{Deserialize, Serialize};
use miot_spec::device::common::emitter::EventType;

#[derive(Clone, Serialize, Deserialize, New)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteSideModuleParam {
    pub ch_id: i64,
    pub url: Url,
}
#[derive(Clone, Serialize, Deserialize, New)]
#[serde(rename_all = "camelCase")]
pub struct BindDeviceModuleParam {
    pub ch_id: i64,
    pub dev_id: String,
}

#[derive(Clone, Serialize, Debug, New)]
#[serde(rename_all = "camelCase")]
pub struct OnDeviceEventParam {
    pub(crate) did: String,
    pub(crate) event: EventType,

}


#[derive(Clone, Serialize, Debug, New)]
#[serde(rename_all = "camelCase")]
pub struct OnCharReadParam {
    // 通道id
    pub ch_id: i64,
    // tag
    pub service_tag: String,

    pub char_tag: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, New)]
#[serde(rename_all = "camelCase")]
pub struct OnCharUpdateParam {
    // 通道id
    pub ch_id: i64,
    // tag
    pub service_tag: String,
    pub char_tag: String,
    pub old_value: JsonValue,
    pub new_value: JsonValue,
}
