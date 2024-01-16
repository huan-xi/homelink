use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use anyhow::anyhow;
use axum::body::HttpBody;
use deno_runtime::deno_core;
use deno_runtime::deno_core::{FsModuleLoader, ModuleSpecifier};
use deno_runtime::deno_core::url::Url;
use deno_runtime::permissions::PermissionsContainer;
use deno_runtime::worker::{MainWorker, WorkerOptions};
use log::info;
use tokio::runtime::Builder;

use tokio::sync::mpsc::Receiver;
use tokio::sync::broadcast;

use tokio::sync::{Mutex, oneshot};
use tokio::time::timeout;
use crate::js_engine::ext::env;
use crate::js_engine::ext::env::{EnvContext};

pub type RespType = i8;
pub type MsgId = u64;

pub enum EngineEvent {
    ExecuteSideModule(Url)
}

pub struct JsEngine {
    sender: tokio::sync::mpsc::Sender<(MsgId, EngineEvent)>,
    resp_recv: Arc<broadcast::Sender<(MsgId, RespType)>>,
    engine_ctrl: Arc<Mutex<oneshot::Sender<u8>>>,
    id: AtomicU64,

}

impl JsEngine {
    pub async fn send_event(&self, event: EngineEvent) -> anyhow::Result<RespType> {
        let id = self.id.fetch_add(1, Ordering::SeqCst);
        self.sender.send((id, event)).await?;
        let mut rx = self.resp_recv.subscribe();
        //等待发送结果
        let resp = timeout(std::time::Duration::from_secs(4), async {
            while let Ok((resp_id, resp)) = rx.recv().await {
                if resp_id == id {
                    info!("js engine recv resp:{:?}", resp);
                    return Ok(resp);
                }
            }
            return Err(anyhow!("js 引擎关闭"));
        }).await.map_err(|f| Err(anyhow!("读取响应超时")))?;
    }
    pub async fn close(&self) {
        let _ = self.engine_ctrl.lock().await.send(1);
    }
}


pub fn init_js_engine(mut context: EnvContext) -> anyhow::Result<JsEngine> {
    // js_tx 控制js 引擎的停止
    let (js_tx, tx_rx) = oneshot::channel::<u8>();
    let (tx, rx) = tokio::sync::mpsc::channel::<(MsgId, EngineEvent)>(10);
    let (resp_tx, _) = tokio::sync::broadcast::channel::<(MsgId, RespType)>(10);

    context.main_channel(Some(tx_rx));

    let context = context.build().unwrap();
    thread::spawn(move || {
        let rt = Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            // let recv = rx;
            init_js_engine0(context, rx).await;
        })
    });
    // resp tx


    Ok(JsEngine {
        sender: tx,
        resp_recv: Arc::new(resp_tx),
        engine_ctrl: Arc::new(Mutex::new(js_tx)),
        id: Default::default(),
    })
}

pub async fn init_js_engine0(context: EnvContext, mut recv: Receiver<(MsgId, EngineEvent)>, resp_sender: Arc<broadcast::Sender<(MsgId, RespType)>>) {
    info!("初始化js 引擎");
    let ops_env = env::deno_env::init_ops_and_esm(context);
    //运行主程序
    let js_path = Path::new("/Users/huanxi/project/home-gateway/data/js_scripts/hello.js");
    let main_module = ModuleSpecifier::from_file_path(js_path).unwrap();
    let mut worker = MainWorker::bootstrap_from_options(
        main_module.clone(),
        PermissionsContainer::allow_all(),
        WorkerOptions {
            module_loader: Rc::new(FsModuleLoader) as Rc<dyn deno_core::ModuleLoader>,
            extensions: vec![ops_env],
            ..Default::default()
        },
    );

    worker.execute_main_module(&main_module).await.unwrap();
    worker.run_event_loop(false).await.unwrap();
    while let Some((msg_id, event)) = recv.recv().await {
        info!("js engine recv event");
        match event {
            EngineEvent::ExecuteSideModule(url) => {
                let res = worker.execute_side_module(&url).await;
                let _ = resp_sender.send((msg_id, 0));
                info!("js engine execute side module result:{:?}", res);
            }
            _ => {}
        }
    };


    info!("js engine main module exit")
}