use std::sync::Arc;

use once_cell::sync::OnceCell;
use sea_orm::DatabaseConnection;

use hap_metadata::metadata::HapMetadata;
use target_hap::hap_manager::HapManage;

use crate::config::cfgs::Configs;
use crate::init::manager::device_manager::IotDeviceManager;

/// 与js 交互的上下文
pub static APP_CONTEXT: OnceCell<ApplicationContext> = OnceCell::new();

pub fn get_app_context() -> &'static ApplicationContext {
    APP_CONTEXT.get().expect("APP_JS_CONTEXT 未初始化")
}

pub fn get_data_dir() -> &'static str {
    let context = get_app_context();
    context.config.server.data_dir.as_str()
}

pub struct ApplicationContext {
    pub config: Configs,
    pub conn: DatabaseConnection,
    // pub js_engine: JsEngine,
    pub hap_metadata: Arc<HapMetadata>,
    pub device_manager: IotDeviceManager,
    pub hap_manager: HapManage,
}