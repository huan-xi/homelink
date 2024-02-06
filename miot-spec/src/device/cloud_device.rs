
use std::time::Duration;
use futures_util::future::BoxFuture;

use crate::device::miot_spec_device::{AsMiotSpecDevice, BaseMiotSpecDevice, DeviceInfo, MiotSpecDevice};
use crate::proto::miio_proto::MiotSpecProtocolPointer;
use crate::proto::protocol::ExitError;


/// 通过云端接入的设备
pub struct MiCloudDevice<T: MiCloudExt> {
    pub base: BaseMiotSpecDevice,
    pub info: DeviceInfo,
    ///协议
    ext: T,
}

//定义一个获取写一个的闭包
pub type MiCloudProtoGetFunc = Box<dyn Fn() -> BoxFuture<'static, Result<MiotSpecProtocolPointer, ExitError>> + Send + Sync + 'static>;


#[async_trait::async_trait]
pub trait MiCloudExt {
    async fn get_proto(&self) -> Result<MiotSpecProtocolPointer, ExitError>;
    async fn register_property(&self, siid: i32, piid: i32);
}

impl<T: MiCloudExt> MiCloudDevice<T> {
    pub fn new(info: DeviceInfo, ext: T) -> Self {
        Self {
            base: Default::default(),
            info,
            ext,
        }
    }
}

impl<T: MiCloudExt + Send + Sync> AsMiotSpecDevice for MiCloudDevice<T> {
    fn as_miot_spec_device(&self) -> Option<&(dyn MiotSpecDevice + Send + Sync)>{
        Some(self)
    }
}

#[async_trait::async_trait]
impl<T: MiCloudExt + Send + Sync> MiotSpecDevice for MiCloudDevice<T> {
    fn get_info(&self) -> &DeviceInfo {
        &self.info
    }

    fn get_base(&self) -> &BaseMiotSpecDevice {
        &self.base
    }

    async fn get_proto(&self) -> Result<MiotSpecProtocolPointer, ExitError> {
        self.ext.get_proto().await
    }
    async fn run(&self) -> Result<(), ExitError> {
        // let proto = self.proto.clone();
        // proto.cloud_client.await;
        //todo 从设备组监听轮询消息,并发布

        loop {
            tokio::time::sleep(Duration::from_secs(100)).await
        }
        Ok(())
    }
    async fn register_property(&self, siid: i32, piid: i32) {
        self.ext.register_property(siid, piid).await
    }
}