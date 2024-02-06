






use crate::device::MiotDevicePointer;



/// 蓝牙数据属性映射协议
/// 蓝牙设备需要通过网关设备去调用
pub struct BleMappingProto {
    gateway: MiotDevicePointer,
}