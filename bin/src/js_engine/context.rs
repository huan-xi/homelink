use once_cell::sync::OnceCell;
use sea_orm::DbConn;
use crate::init::manager::device_manager::IotDeviceManager;
use crate::init::manager::hap_manager::HapManage;


/// 与js 交互的上下文
pub static APP_JS_CONTEXT: OnceCell<AppJsContext> = OnceCell::new();

pub fn get_app_js_context() -> &'static AppJsContext {
    APP_JS_CONTEXT.get().expect("APP_JS_CONTEXT 未初始化")
}

pub struct AppJsContext {
    pub conn: DbConn,
    pub dev_manager: IotDeviceManager,
    pub hap_manager: HapManage,
}