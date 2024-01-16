use std::fs;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc};
use dashmap::DashMap;
use deno_runtime::deno_fetch::reqwest::Url;
use futures_util::FutureExt;
use log::{error, info};
use sea_orm::JsonValue;
use tap::TapFallible;
use miot_spec::device::miot_spec_device::{MiotSpecDevice};
use miot_spec::device::MiotDevicePointer;
use crate::config::context::get_app_context;
use crate::init::{DevicePointer};
use crate::js_engine::init_js_engine::EngineEvent;

pub struct JsModule {}

#[derive(Clone)]
pub struct DeviceWithJsEngine {
    device: MiotDevicePointer,
    /// 特征 cid 和 对应的module
    mapping_js_map: Arc<DashMap<i64, JsModule>>,
}

impl Deref for DeviceWithJsEngine {
    type Target = Arc<dyn MiotSpecDevice + Send + Sync>;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

impl DeviceWithJsEngine {
    pub fn new(device: Arc<dyn MiotSpecDevice + Send + Sync>) -> Self {
        Self {
            device,
            mapping_js_map: Default::default(),
        }
    }

    /// 初始化js模块,获取通道
    pub fn init_js_module() {}
    /// 执行js,与js通信
    pub async fn eval_js(&self, cid: i64, script: &str) -> anyhow::Result<()> {
        let a = self.mapping_js_map.get_mut(&cid);
        if a.is_some() {
            return Ok(());
        }
        //输出到文件
        let context = get_app_context();
        let dir = context.config.server.data_dir.as_str();
        let js_file = PathBuf::from(format!("{}/js_scripts/mapping/mapping_{}.js", dir, cid));
        fs::create_dir_all(js_file.parent().unwrap())?;
        fs::write(js_file.as_path(), script)?;
        let url = Url::parse(js_file.as_path().to_str()?)?;

        context.js_engine.send_event(EngineEvent::ExecuteSideModule(url)).await
            .tap_err(|e| error!("执行js错误"))?;


        todo!();
        /*   let source = Source::from_bytes(js_scripts.as_str());
           let mut js_engine_lock = self.js_engine
               .lock()
               .map_err(|e| anyhow!("获取js引擎失败:{:?}", e))?;

           let val = match js_engine_lock.as_mut() {
               None => {
                   //初始化js引擎

                   // let context = boa_engine::Context::default();
                   let context = init_js_engine(self.device.clone());
                   js_engine_lock.replace(context);
                   js_engine_lock.as_mut().unwrap()
               }
               Some(js_engine) => {
                   js_engine
               }
           }.eval(source).map_err(|e| anyhow!("js执行错误:{:?}", e))?;
           drop(js_engine_lock);
           json_value_from_js_value(val)*/
    }

    /*pub async fn get_js_engine(&self) -> boa_engine::Context<'static> {
        //不存在就初始化
        let js_engine = self.js_engine.read().await;
        if js_engine.is_none() {
            drop(js_engine);
            let mut js_engine = self.js_engine.write().await;
            if js_engine.is_none() {
                info!("初始化设备:{}js引擎", self.device.get_info().did);
                let js_engine = boa_engine::Context::default();
                *js_engine = Some(js_engine);
            }
        } else {

        }
    }*/
}


pub struct DeviceTask {
    dev: DevicePointer,
    close_sender: tokio::sync::oneshot::Sender<bool>,
    // exit_recv: tokio::sync::oneshot::Receiver<bool>,
}


pub struct IotDeviceManagerInner {
    device_map: dashmap::DashMap<i64, DeviceTask>,
}


impl IotDeviceManagerInner {
    pub fn new() -> Self {
        Self {
            device_map: dashmap::DashMap::new()
        }
    }
    pub fn push_device(&self, device_id: i64, device: DevicePointer) {
        let dev_c = device.clone();
        let (close_sender, recv) = tokio::sync::oneshot::channel();

        let device = DeviceTask {
            dev: device,
            close_sender,
        };
        self.device_map.insert(device_id, device);
        //执行任务
        tokio::spawn(async move {
            let task = async move {
                loop {
                    let res = dev_c.run().await;
                    error!("设备连接断开:{:?},res:{:?}", dev_c.get_info().did, res);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    info!("设备重连:{}", dev_c.get_info().did);
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

    /// 关闭设备之前要移除 所有hap 设备，hap设备会有arc 的应用
    pub async fn remove_device(&self, device_id: i64) -> anyhow::Result<()> {

        // 发送移除指令
        match self.device_map.remove(&device_id) {
            None => {
                return Err(anyhow::anyhow!("设备不存在"));
            }
            Some((id, task)) => {
                let res = task.close_sender.send(true);
                //移除hap 设备

                /*let sender = task.value().sender.clone();
                sender.send(true).await?;
                self.device_map.remove(&device_id);*/
            }
        };
        Ok(())
        // 等待设备停止成功
    }

    /// 关闭
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
    pub fn new() -> Self { Self { inner: Arc::new(IotDeviceManagerInner::new()) } }
}

impl Deref for IotDeviceManager {
    type Target = IotDeviceManagerInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}