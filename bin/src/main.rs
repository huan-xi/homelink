#![allow(unused_variables)]

use std::sync::Arc;
use axum::Router;
use log::{error, info};
use tokio::select;
use tokio::sync::oneshot;
use tower_http::services::ServeDir;

use lib::api::router;
use lib::api::state::AppState;
use lib::config::cfgs::Configs;
use lib::config::context::{APP_CONTEXT, ApplicationContext, get_app_context};
use lib::db::init::{db_conn, migrator_up};
use lib::init;
use lib::init::device_init::init_iot_device_manager;
use lib::init::manager::device_manager::IotDeviceManager;
use lib::init::manager::hap_manager::HapManage;
use lib::init::manager::mi_account_manager::MiAccountManager;
use lib::init::manager::template_manager::TemplateManager;
use lib::init::{logger_init, Managers};
use lib::js_engine::context::EnvContext;
use lib::js_engine::init_js_engine::init_js_engine;
use lib::migration::Migrator;
use hap_metadata::hap_metadata;


/// 先创建http服务
/// 创建homekit 服务
/// 创建米家设备
/// 映射米家设备到homekit设备
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Err(e) = log4rs::init_file("log4rs1.yaml", Default::default()) {
        // 初始化默认日志
        logger_init::init_logger();
    }
    //读取配置
    let config = Configs::init();
    let addr = config.server.address.parse().expect("监听地址解析失败");
    // let addr = SocketAddr::new([0, 0, 0, 0].into(), 5514);
    let conn = db_conn(&config.server).await;
    //数据库版本迁移
    migrator_up(&conn).await;

    let hap_metadata = Arc::new(hap_metadata()?);
    // 初始化hap 服务器
    let iot_device_manager = IotDeviceManager::new();
    let hap_manager = HapManage::new(hap_metadata.clone());
    let template_manager = TemplateManager::new();


    let mi_account_manager = MiAccountManager::new(conn.clone());

    let app_state = AppState::new(conn.clone(),
                                  hap_metadata.clone(),
                                  iot_device_manager.clone(),
                                  hap_manager.clone(),
                                  mi_account_manager.clone(),
                                  template_manager.clone());

    let app = Router::new()
        .nest_service("/", ServeDir::new("dist/"))
        .nest("/api", router::api())
        .with_state(app_state.clone());

    //初始化js 引擎
    let js_engine = init_js_engine(config.server.data_dir.clone(), EnvContext {
        info: "home gateway".to_string(),
        version: "v1.0.0".to_string(),
        #[cfg(feature = "deno")]
        main_recv: None,
        conn: conn.clone(),
        dev_manager: iot_device_manager.clone(),
        hap_manager: hap_manager.clone(),
    }).await?;


    let res = APP_CONTEXT.set(ApplicationContext {
        config,
        conn: conn.clone(),
        js_engine,
        hap_metadata: hap_metadata.clone(),
        device_manager: iot_device_manager.clone(),
        hap_manager: hap_manager.clone(),
    });
    if res.is_err() {
        panic!("APP_CONTEXT 初始化失败");
    }
    let context = get_app_context();


    let api_server =
        axum::Server::bind(&addr)
            .serve(app.into_make_service());
    info!("api_server start at :{:?}", addr);
    let (api_server_ch_send, api_server_ch) = oneshot::channel::<bool>();
    tokio::spawn(async move {
        if let Err(e) = api_server.await {
            error!("api_server error:{:?}", e);
        }
        // 发送到通道
        let _ = api_server_ch_send.send(false);
    });

    // 初始化hap设备
    let manager = Managers {
        hap_manager: hap_manager.clone(),
        iot_device_manager: iot_device_manager.clone(),
        mi_account_manager: mi_account_manager.clone(),
    };
    // 初始化模板
    template_manager.init().await?;
    // 初始化iot设备
    init_iot_device_manager(&conn, iot_device_manager.clone(), mi_account_manager.clone()).await?;
    // 初始化hap 设备
    init::hap_init::init_hap_list(&conn, hap_manager.clone(), iot_device_manager.clone()).await?;

    // 等待引擎退出
    // let recv = context.js_engine.resp_recv.subscribe();
    /* let engin_statue = async {
         while let Ok(event_type) = recv.recv().await {
             if let EngineEventResp::Exit(str) = event_type {
                 error!("js engine exit:{:?}", str);
                 break;
             }
         }
     };*/

    loop {
        select! {
            // _ = engin_statue=> break,
            _ = api_server_ch => {
                info!("api server recv resp");
                break;
            }
        }
    }

    // api_server_ch.await?;
    context.js_engine.close().await;
    // 服务停止
    drop(app_state);
    // let _ = js_tx.send(0);
    iot_device_manager.close().await;
    hap_manager.close().await;
    Ok(())
}

