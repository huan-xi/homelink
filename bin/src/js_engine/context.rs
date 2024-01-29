use sea_orm::DbConn;
use crate::init::manager::device_manager::IotDeviceManager;
use crate::init::manager::hap_manager::HapManage;

pub struct EnvContext {
    pub info: String,
    pub version: String,
    #[cfg(feature = "deno")]
    pub main_recv: Option<crate::js_engine::channel::main_channel::ModuleRecv>,
    pub conn: DbConn,
    pub dev_manager: IotDeviceManager,
    pub hap_manager: HapManage,
    // 特征与模块通道的映射
    // 和context_js中的map 是同一个,
    // pub mapping_characteristic_map: Arc<DashMap<i64, MappingCharacteristicRecv>>,
    // pub hap_module_map: HapAccessoryModuleMapPointer,
}
