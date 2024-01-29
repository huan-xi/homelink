use std::ops::Deref;
use std::sync::Arc;

use axum::body::HttpBody;
use dashmap::DashMap;
use dashmap::mapref::entry::Entry;
use futures_util::FutureExt;
use log::{error, info};
use tokio::sync::oneshot;

use miot_spec::device::common::emitter::EventType;
use miot_spec::device::miot_spec_device::MiotSpecDevice;
use miot_spec::device::MiotDevicePointer;

use crate::config::context::get_app_context;
use crate::init::DevicePointer;

#[cfg(feature = "deno")]
use crate::js_engine::channel::{
    params::OnDeviceEventParam,
    main_channel::ToModuleEvent
};

pub struct JsModule {
    ///与 特征脚本通信的通道
    // pub sender: Arc<MappingCharacteristicSender>,
    /// module 的关闭通道
    pub exit_channel: oneshot::Sender<u8>,

}

#[derive(Clone)]
pub struct DeviceWithJsEngine {
    pub(crate) device: MiotDevicePointer,
    /// 特征 cid 和 对应的module
    /// 一个设备运行多个特征脚本
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
    pub async fn init_mapping_js_module(&self, cid: i64, script: &str, force: bool) -> anyhow::Result<()> {
        todo!();
        /*  let chanel_sender = self.mapping_js_map.get_mut(&cid);
          if let Some(sender) = chanel_sender.as_ref() {
              //todo force 处理
              return Ok(sender.sender.clone());
          }

          //输出到文件
          let context = get_app_context();
          let dir = context.config.server.data_dir.as_str();
          let js_file_str = format!("{}/js_scripts/mapping/mapping_{}.js", dir, cid);
          let js_file = PathBuf::from(js_file_str.as_str());
          fs::create_dir_all(js_file.parent().unwrap())?;
          fs::write(js_file.as_path(), script)?;
          let url = Url::parse(js_file_str.as_str())?;

          //注册mapping 通道,
          let channel = context.js_engine.mapping_characteristic_map.get(&cid);
          let (tx, _) = broadcast::channel(10);
          let js_to_tx = Arc::new(tx);
          // 注册模块
          let (recv, exit, sender) = MappingCharacteristicRecv::new(js_to_tx.clone());
          let ch_sender = MappingCharacteristicSender::new(sender, js_to_tx.clone());
          let ch_sender = Arc::new(ch_sender);
          context.js_engine.mapping_characteristic_map.insert(cid, recv);
          let module = JsModule { sender: ch_sender.clone(), exit_channel: exit };

          return match context.js_engine
              .execute_side_module(      ExecuteSideModuleParam::new(1, url)).await
              .tap_err(|e| error!("执行js错误")) {
              Ok(_) => {
                  self.mapping_js_map.insert(cid, module);
                  Ok(ch_sender.clone())
              }
              Err(_) => {
                  //删除掉
                  context.js_engine.mapping_characteristic_map.remove(&cid);
                  Err(anyhow::anyhow!("执行js错误"))
              }
          };*/
    }









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
    /// 关闭整个任务
    close_sender: oneshot::Sender<bool>,
    // exit_recv: tokio::sync::oneshot::Receiver<bool>,
}


pub struct IotDeviceManagerInner {
    device_map: DashMap<i64, DeviceTask>,
    did_map: DashMap<String, i64>,
    device_id_enable_js_map: DashMap<i64, bool>,
}


impl IotDeviceManagerInner {
    pub fn new() -> Self {
        Self {
            device_map: DashMap::new(),
            did_map: Default::default(),
            device_id_enable_js_map: Default::default(),
        }
    }
    pub fn push_device(&self, device_id: i64, device: DevicePointer) {
        let dev_c = device.clone();
        let (close_sender, recv) = oneshot::channel();

        let device = DeviceTask {
            dev: device,
            close_sender,
        };
        self.device_map.insert(device_id, device);
        self.did_map.insert(dev_c.get_info().did.clone(), device_id);
        //执行任务
        tokio::spawn(async move {
            let task = async move {
                loop {
                    let res = dev_c.run().await;
                    //标记重试次数+1
                    let incr = dev_c.get_base().retry_info.incr().await;
                    let interval = dev_c.get_base().retry_info.get().await;
                    error!("设备连接断开:{:?},res:{:?},等待{interval}毫秒后,第{incr}次重试", dev_c.get_info().did, res);
                    tokio::time::sleep(tokio::time::Duration::from_millis(interval as u64)).await;
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

    pub fn is_running(&self, id: i64) -> bool {
        self.device_map.contains_key(&id)
    }
    pub fn stop_device(&self, id: i64) -> anyhow::Result<()> {
        let dev = self.device_map.remove(&id);
        if let Some((id,task)) = dev{
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

            let listener = Box::new(move |event: EventType| {
                let did = did.to_string();
                let sender = sender.clone();
                async move {
                    //排除掉网关消息
                    if let EventType::GatewayMsg(_) = event {
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