use crate::db::entity::iot_device::IotDeviceType;

/// 配件模板
pub struct AccessoryTemplate {
    /// 配件的标识
    pub acc_id: String,
    /// 配件的版本
    pub acc_version: String,
    /// 配件的类型
    pub acc_type: String,
    /// 配件的模型
    pub model: String,
    /// 配件的模型版本
    pub version: String,
    /// 配件的设备类型
    pub device_type: IotDeviceType,
    /// 配件的属性
    pub properties: Vec<String>,
    /// 配件的事件
    pub events: Vec<String>,
    /// 配件的方法
    pub actions: Vec<String>,
}

/// iot device 转模板
/// 一个米家设备 对应多个iot device模板
/// 一个iot device 对应多个配件，配件必须基于桥接器存在
pub struct IotDeviceTemplate {
    ///模板的标识
    pub temp_id: String,
    /// 模板的版本
    pub temp_version: String,
    /// 模型
    pub model: String,
    /// 模型版本
    pub version: Option<String>,
    /// 设备类型
    pub device_type: IotDeviceType,
    /// 配件
    pub accessories: Vec<AccessoryTemplate>,
}

#[test]
pub fn test() {
// iotdeviceen
}