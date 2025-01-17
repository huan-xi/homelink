use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anyhow::anyhow;
use axum::body::HttpBody;
use log::{error, info};
use tap::TapFallible;
use tokio::runtime::Builder;
use tokio::time::timeout;

use crate::js_engine::channel::main_channel;
use crate::js_engine::channel::main_channel::{FromModuleResp, ResultSenderPointer, ToModuleEvent, ToModuleSender};
use crate::js_engine::channel::params::ExecuteSideModuleParam;
use crate::js_engine::context::EnvContext;


#[derive(Clone)]
pub struct JsEngine {
    /// 向引擎发送事件，比如执行模块
    sender: Arc<ToModuleSender>,
    ///引擎发出广播
    pub resp_recv: ResultSenderPointer,
    // engine_ctrl: Arc<Mutex<Option<oneshot::Sender<u8>>>>,
    // id: AtomicU64,
}

impl JsEngine {
    pub async fn send(&self, event: ToModuleEvent) -> anyhow::Result<FromModuleResp> {
        self.sender.send(event).await
    }

    pub async fn execute_side_module(&self, param: ExecuteSideModuleParam) -> anyhow::Result<()> {
        let id = param.ch_id;

        let resp = self.sender.send(ToModuleEvent::ExecuteSideModule(param))
            .await
            .tap_err(|e| error!("发送ExecuteSideModule 指令失败:{:?}",e))?;
        return if let FromModuleResp::ExecuteModuleResp(resp) = resp {
            if resp.ch_id != id {
                return Err(anyhow!("返回的通道id不一致"));
            }
            Ok(())
        } else {
            Err(anyhow!("返回的不是ExecuteModuleResp"))
        };
    }
    pub async fn close(&self) {
        /* let _ = self.engine_ctrl.clone()
             .lock()
             .await
             .take()
             .map(|tx| tx.send(1));*/
    }
}


#[cfg(feature = "deno")]
pub async fn init_js_engine(data_dir: String, mut context: EnvContext) -> anyhow::Result<JsEngine> {
    // js_tx 控制js 引擎的停止
    let (to_module_sender, recv) = main_channel::channel();
    context.main_recv = Some(recv);

    let resp_recv = to_module_sender.read_result_recv.clone();
    let resp_recv_c = resp_recv.clone();
    thread::spawn(move || {
        let rt = Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            // let recv = rx;
            let result = init_js_engine0(data_dir.as_str(), context, resp_recv_c).await;
            let msg = match result {
                Ok(_) => "正常退出".to_string(),
                Err(e) => e.to_string(),
            };
            // let _ = resp_recv_c1.send(EngineEventResp::Exit(format!("{:?}", msg)));
        });
    });
    let resp_recv_c = resp_recv.clone();
    // 等待js 引擎启动
    let js_result = timeout(Duration::from_secs(4), async {
        while let Ok((id, event_type)) = resp_recv_c.subscribe().recv().await {
            match event_type {
                FromModuleResp::Success => {
                    return Ok(());
                }
                FromModuleResp::EnginExit(str) => {
                    error!("js engine exit:{:?}", str);
                    return Err(anyhow!("js engine exit"));
                }
                _ => {}
            }
        }
        return Err(anyhow!("js engine exit"));
    }).await.map_err(|f| anyhow!("js 启动超时"))?;

    js_result?;

    /*  while let Ok(s) = resp_recv.subscribe().recv().await {
          info!("js engine recv resp:{:?}", s);
      };*/
    Ok(JsEngine {
        sender: to_module_sender,
        resp_recv,
        // engine_ctrl: Arc::new(Mutex::new(Some(js_tx))),
        // id: Default::default(),
    })
}

/// 初始化js 引擎
pub async fn init_js_engine0(data_dir: &str, context: EnvContext, resp_sender: ResultSenderPointer) -> anyhow::Result<()> {
    info!("初始化js 引擎");

    //运行主程序
    let dir = format!("{}/js_scripts", data_dir);
    let js_file_str = format!("{}/main.js", dir);
    let js_path = PathBuf::from(js_file_str);


    // #[cfg(feature = "deno")]
    crate::js_engine::deno_runtime::start_deno_runtime(context, dir, js_path).await?;


    let _ = resp_sender.send((0, FromModuleResp::EnginExit("end".to_string())));
    error!("js engine main module exit");
    Ok(())
}