use crate::init::manager::template_manager::SourcePlatformModel;
use crate::template::hl_template::TemplateFormat;
use serde_aux::prelude::deserialize_number_from_string;
#[derive(serde::Deserialize, Debug)]
pub struct ApplyTemplateParam {
    pub(crate) text: String,
    pub(crate) format: TemplateFormat,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub(crate) bridge_id: i64,
    pub device_id: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct CheckTemplateParam {
    pub(crate) text: String,
    pub(crate) format: TemplateFormat,
    pub source_id: String,
    pub source_platform: String,
}



