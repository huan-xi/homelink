// hap读取数据,需要返回数据值
const on_read = async (context) => {
    console.log('on_read');
}
// hap 设置数据
const on_update = async (context, old_val, new_val) => {

}
// 收到设备数据
const on_device_event = async (context, event) => {

}

export default {
    on_read,
    on_update,
    on_device_event
}