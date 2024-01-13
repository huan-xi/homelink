use boa_engine::Context;
use log::info;
use miot_spec::device::MiotDevicePointer;

pub fn init_js_engine(dev_ptr: MiotDevicePointer) -> Context<'static> {
    info!("初始化设备:{}js引擎", dev_ptr.get_info().did);
    let mut context = Context::default();
    //注册函数


    return context;
}
