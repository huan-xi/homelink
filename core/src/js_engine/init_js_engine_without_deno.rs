use crate::js_engine::context::EnvContext;

#[derive(Clone)]
pub struct JsEngine {}


impl JsEngine {}

impl JsEngine {
    pub async fn close(&self) {}
}

pub async fn init_js_engine(data_dir: String, mut context: EnvContext) -> anyhow::Result<JsEngine> {
    Ok(JsEngine {})
}