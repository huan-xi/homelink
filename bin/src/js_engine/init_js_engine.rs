use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{panic, thread};
use anyhow::anyhow;
use axum::body::HttpBody;
use dashmap::DashMap;
use deno_runtime::deno_core;
use deno_runtime::deno_core::{FsModuleLoader, ModuleId, ModuleSpecifier};
use deno_runtime::deno_core::error::AnyError;
use deno_runtime::deno_core::url::Url;
use deno_runtime::permissions::PermissionsContainer;
use deno_runtime::worker::{MainWorker, WorkerOptions};
use log::{error, info};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::runtime::Builder;

use tokio::sync::mpsc::Receiver;
use tokio::sync::broadcast;

use tokio::sync::{Mutex, oneshot};
use tokio::time::timeout;
use crate::config::context::{get_app_context, get_data_dir};
use crate::js_engine::channel::hap_channel::HapAccessoryModuleRecv;
use crate::js_engine::channel::mapping_characteristic_channel::{MappingCharacteristicRecv, MappingCharacteristicSender};
use crate::js_engine::ext::env;
use crate::js_engine::ext::env::{EnvContext};
use crate::js_engine::init_js_engine::EngineEventResp::ExecuteSideModuleResult;
use crate::js_engine::scripts::ScriptAsset;

#[derive(Debug, Clone)]
pub enum EngineEventResp {
    /// 成功与否?
    ExecuteSideModuleResult((i64,bool)),

    /// 模块退出
    SideModuleExit(i64),

    Success,
    Exit(String),
}

pub enum EngineEvent {
    ExecuteSideModule(ExecuteSideModuleParam)
}

pub type MsgId = u64;

pub struct ExecuteSideModuleParam {
    pub ch_id: i64,
    pub url: Url,
}

impl ExecuteSideModuleParam {
    pub fn new(ch_id: i64, url: Url) -> Self {
        Self {
            ch_id,
            url,
        }
    }
}


pub type EngineEventSender = tokio::sync::mpsc::Sender<EngineEvent>;
pub type EngineEventRecvPointer = Arc<broadcast::Sender<EngineEventResp>>;

pub struct JsEngine {
    /// 向引擎发送事件，比如执行模块
    sender: tokio::sync::mpsc::Sender<EngineEvent>,
    ///引擎发出广播
    pub resp_recv: EngineEventRecvPointer,
    engine_ctrl: Arc<Mutex<Option<oneshot::Sender<u8>>>>,
    id: AtomicU64,
    pub mapping_characteristic_map: Arc<DashMap<i64, MappingCharacteristicRecv>>,
    pub hap_map: Arc<DashMap<i64, HapAccessoryModuleRecv>>,
}

impl JsEngine {
    pub async fn execute_side_module(&self, param: ExecuteSideModuleParam) -> anyhow::Result<bool> {
        let id = param.ch_id;

        self.sender.send(EngineEvent::ExecuteSideModule(param)).await?;
        let mut rx = self.resp_recv.subscribe();
        //等待发送结果
        let resp = timeout(std::time::Duration::from_secs(4), async {
            while let Ok((ExecuteSideModuleResult((resp_id, resp)))) = rx.recv().await {
                if resp_id == id {
                    info!("js engine recv resp:{:?}", resp);
                    return Ok(resp);
                }
            };
            return Err(anyhow!("js 引擎关闭"));
        }).await.map_err(|f| anyhow!("读取响应超时"))?;
        resp
    }
    pub async fn close(&self) {
        let _ = self.engine_ctrl.clone()
            .lock()
            .await
            .take()
            .map(|tx| tx.send(1));
    }
}


pub fn init_js_engine(data_dir: String, mut context: EnvContext) -> anyhow::Result<JsEngine> {
    // js_tx 控制js 引擎的停止
    let (js_tx, tx_rx) = oneshot::channel::<u8>();
    context.main_channel.replace(tx_rx);
    // 用于执行引擎事件
    let (tx, rx) = tokio::sync::mpsc::channel::<EngineEvent>(10);

    // 用于引擎广播响应
    let (resp_recv, _) = broadcast::channel::<EngineEventResp>(10);
    let resp_recv = Arc::new(resp_recv);
    let resp_recv_c = resp_recv.clone();
    let resp_recv_c1 = resp_recv.clone();

    let mapping_characteristic_map = context.mapping_characteristic_map.clone();
    let hap_map = context.hap_module_map.clone();
    thread::spawn(move || {
        let rt = Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            // let recv = rx;
            let result = init_js_engine0(data_dir.as_str(), context, rx, resp_recv_c).await;

            let msg = match result {
                Ok(_) => "正常退出".to_string(),
                Err(e) => e.to_string(),
            };
            let _ = resp_recv_c1.send(EngineEventResp::Exit(format!("{:?}", msg)));
        });
        /*       match handle.blocking_wait() {
                   Ok(_) => println!("No panic occurred."),
                   Err(e) => {
                       error!("panic occurred: {:?}", e);
                       let _ = resp_recv.send((MsgId::MAX, RespType::Exit(format!("{:?}", e))));
                   }
               }*/
    });
    // resp tx


    Ok(JsEngine {
        sender: tx,
        resp_recv,
        engine_ctrl: Arc::new(Mutex::new(Some(js_tx))),
        id: Default::default(),
        mapping_characteristic_map,
        hap_map,
    })
}

pub async fn init_js_engine0(data_dir: &str, context: EnvContext, mut recv: Receiver<EngineEvent>, resp_sender: Arc<broadcast::Sender<EngineEventResp>>) -> anyhow::Result<()> {
    info!("初始化js 引擎");
    let ops_env = env::deno_env::init_ops_and_esm(context);
    get_data_dir();
    //运行主程序
    let dir = format!("{}/js_scripts", data_dir);
    let js_file_str = format!("{}/main.js", dir);
    let js_path = PathBuf::from(js_file_str);
    if let Err(e) = fs::metadata(js_path.as_path()).await {
        let embed_file = ScriptAsset::get("main.js")
            .unwrap();
        fs::create_dir_all(js_path.parent().unwrap()).await?;
        let mut file = fs::File::open(js_path.as_path()).await?;
        file.write_all(embed_file.data.as_ref()).await?;
    }

    let main_module = ModuleSpecifier::from_file_path(js_path.as_path())
        .map_err(|e| anyhow!("main.js 不存在"))?;

    let mut worker = MainWorker::bootstrap_from_options(
        main_module.clone(),
        PermissionsContainer::allow_all(),
        WorkerOptions {
            module_loader: Rc::new(FsModuleLoader) as Rc<dyn deno_core::ModuleLoader>,
            extensions: vec![ops_env],
            ..Default::default()
        },
    );


    worker.execute_main_module(&main_module).await?;
    worker.run_event_loop(false).await?;

    let handle_recv = async {
        while let Some(event) = recv.recv().await {
            info!("js engine recv event");
            match event {
                EngineEvent::ExecuteSideModule(param) => {
                    let pid = param.ch_id;
                    match worker.preload_side_module(&param.url).await {
                        Ok(id) => {
                            let mut receiver = worker.js_runtime.mod_evaluate(id);
                            let _ = resp_sender.send(EngineEventResp::ExecuteSideModuleResult((pid, true)));
                            let s1 = resp_sender.clone();
                            tokio::spawn(async move {
                                let res = receiver.await;
                                let _ = s1.send(EngineEventResp::SideModuleExit(param.ch_id));
                                // info!("js engine execute side module result:{:?}", res);
                            });
                        }
                        Err(err) => {
                            error!("preload_side_module js engine execute side module error:{:?}", err);

                            // 模块执行失败
                            let _ = resp_sender.send(EngineEventResp::ExecuteSideModuleResult((pid, false)));
                        }
                    };
                }
            }
        };
    };

    loop {
        tokio::select! {
            _ = handle_recv => break,
        }
    }


    info!("js engine main module exit");

    Ok(())
}