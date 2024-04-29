mod init;
mod mijia;
mod native_ble;

use std::ops::Deref;
use std::sync::Arc;
use axum::body::HttpBody;
use dashmap::DashMap;
use dashmap::mapref::entry::Entry;
use log::{error, info};
use sea_orm::DatabaseConnection;
use tokio::sync::oneshot;

use miot_proto::device::common::emitter::MijiaEvent;
use miot_proto::device::miot_spec_device::MiotSpecDevice;
use crate::config::context::get_app_context;
use crate::init::DevicePointer;
use crate::init::manager::ble_manager::BleManager;
use crate::init::manager::mi_account_manager::MiAccountManager;


pub struct DeviceTask {
    dev: DevicePointer,
    /// 关闭整个任务
    close_sender: oneshot::Sender<bool>,
}


pub struct IotDeviceManagerInner {
    device_map: DashMap<i64, DeviceTask>,
    did_map: DashMap<String, i64>,
    device_id_enable_js_map: DashMap<i64, bool>,
    conn: DatabaseConnection,
    mi_account_manager: MiAccountManager,
    ble_manager: BleManager,
}


impl IotDeviceManagerInner {

    pub fn push_device(&self, device_id: i64, device: DevicePointer) {
        let dev_c = device.clone();
        let (close_sender, recv) = oneshot::channel();

        let device = DeviceTask {
            dev: device,
            close_sender,
        };
        self.device_map.insert(device_id, device);
        //todo  self.did_map.insert(dev_c.get_info().did.clone(), device_id);
        //执行任务
        tokio::spawn(async move {
            let task = async move {
                loop {
                    let res = dev_c.run().await;
                    //标记重试次数+1
                    let incr = dev_c.retry_info().incr().await;
                    let interval = dev_c.retry_info().get().await;
                    error!("设备连接断开:{:?},res:{:?},等待{interval}毫秒后,第{incr}次重试", dev_c.dev_id(), res);
                    tokio::time::sleep(tokio::time::Duration::from_millis(interval as u64)).await;
                    info!("设备重连:{}", dev_c.dev_id());
                }
            };
            loop {
                tokio::select! {
                    _= recv=>{
                        info!("设备任务退出:{}",device_id);
                        break
                    }
                    _= task=>{break}
                }
            }
        });
    }
    pub fn get_device(&self, device_id: i64) -> Option<DevicePointer> {
        self.device_map.get(&device_id).map(|i| i.value().dev.clone())
    }

    /// 关闭设备之前要移除 所有hap 设备，hap设备会有arc 的引用
    pub async fn remove_device(&self, device_id: i64) -> anyhow::Result<()> {

        // 发送移除指令
        if let Some((id, task)) = self.device_map.remove(&device_id) {
            let _ = task.close_sender.send(true);
        }
        // 等待设备停止成功
        Ok(())
    }

    /// 关闭管理器
    pub async fn close(&self) {
        /*let device_handlers: Vec<BoxFuture<()>> = self.device_map
            .iter()
            .map(|i| {
                let dev = i.value().clone();
                async move {
                    let res = dev.run().await;
                    error!("设备连接断开:{:?},res:{:?}", dev.get_info().did,res);
                }.boxed()
            }).collect();
        join_all(device_handlers).await;*/
    }
}


#[derive(Clone)]
pub struct IotDeviceManager {
    inner: Arc<IotDeviceManagerInner>,
}

impl IotDeviceManager {
    pub fn new(conn: DatabaseConnection,
               mi_account_manager: MiAccountManager,
               ble_manager: BleManager,
    ) -> Self {
        Self {
            inner: Arc::new(
                IotDeviceManagerInner {
                    device_map: Default::default(),
                    did_map: Default::default(),
                    device_id_enable_js_map: Default::default(),
                    conn,
                    mi_account_manager,
                    ble_manager,
                }
            )
        }
    }

    pub fn is_running(&self, id: i64) -> bool {
        self.device_map.contains_key(&id)
    }
    pub fn stop_device(&self, id: i64) -> anyhow::Result<()> {
        let dev = self.device_map.remove(&id);
        if let Some((id, task)) = dev {
            let _ = task.close_sender.send(true);
        }
        Ok(())
    }


    /// 开启js 提交监听器
    #[cfg(feature = "deno")]
    pub async fn enable_to_js(&self, did: &str) -> anyhow::Result<()> {
        let dev = self.did_map.get(did)
            .and_then(|i| self.device_map.get(i.value()))
            .ok_or(anyhow::anyhow!("设备不存在"))?;

        if let Entry::Vacant(entry) = self.device_id_enable_js_map.entry(*dev.key()) {
            let sender = get_app_context().js_engine.clone();
            info!("开启js 通道:{did}");
            let did = did.to_string();

            let listener = Box::new(move |event: MijiaEvent| {
                let did = did.to_string();
                let sender = sender.clone();
                async move {
                    //排除掉网关消息
                    if let MijiaEvent::GatewayMsg(_) = event {
                        return Ok(());
                    };
                    let _ = sender.send(ToModuleEvent::OnDeviceEvent(OnDeviceEventParam {
                        did: did.to_string(),
                        event,
                    })).await;
                    Ok(())
                }.boxed()
            });


            dev.dev.add_listener(listener).await;
            entry.insert(true);
        };


        Ok(())
    }
}

impl Deref for IotDeviceManager {
    type Target = IotDeviceManagerInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}