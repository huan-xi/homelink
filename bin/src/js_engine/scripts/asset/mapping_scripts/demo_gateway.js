// hap ch read data,require return value
const on_read = async () => {

    return 1;
}
// hap set data
const on_update = async (context, old_val, new_val) => {
    console.log("on_update", old_val, new_val)
}
// 收到设备数据
const on_device_event = async (context, event) => {
    console.log("on_device_event", event)
}

export default {
    on_read,
    on_update,
    on_device_event
}