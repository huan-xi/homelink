use crate::template::hl_template::TemplateFormat;

#[derive(serde::Deserialize, Debug)]
pub struct CheckTemplateParam {
    pub(crate) text: String,
    pub(crate) format: TemplateFormat,
    pub source_id: String,
    pub source_platform: String,
}


