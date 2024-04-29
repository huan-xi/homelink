#![allow(unused_variables)]

use std::sync::Arc;
use axum::{Extension, Router};
use log::{error, info};
use tokio::sync::{Mutex, oneshot};
use tower_http::services::ServeDir;
use hl_integration::integration::HlSourceIntegrator;
use lib::api::router;
use lib::api::state::{AppState, ServerShutdownSignal};
use lib::config::cfgs::Configs;
use lib::config::context::{APP_CONTEXT, ApplicationContext, get_app_context};
use lib::db::init::{db_conn, migrator_up};
use lib::init::manager::device_manager::IotDeviceManager;
use lib::init::manager::mi_account_manager::MiAccountManager;
use lib::init::manager::template_manager::TemplateManager;
use lib::init::{logger_init, Managers};
// use lib::js_engine::context::EnvContext;
// use lib::js_engine::init_js_engine::init_js_engine;
use hap_metadata::hap_metadata;
use lib::init::hap_init::init_hap_list;
use lib::init::manager::ble_manager::BleManager;
use lib::socketio::socket_io_layer;
use target_hap::hap_manager::HapManage;
use xiaomi_integration::integration::XiaomiIntegration;


/// 先创建http服务
/// 创建homekit 服务
/// 创建米家设备
/// 映射米家设备到homekit设备
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化默认日志
    logger_init::init_logger();

    //读取配置
    let config = Configs::init();
    let addr = config.server.address.clone();

    // let addr = SocketAddr::new([0, 0, 0, 0].into(), 5514);
    let conn = db_conn(&config.server).await;
    //数据库版本迁移
    migrator_up(&conn).await;
    let ble_manager = BleManager::new();
    ble_manager.init().await;
    let hap_metadata = Arc::new(hap_metadata()?);
    let mi_account_manager = MiAccountManager::new(conn.clone());
    let template_manager = TemplateManager::new();
    // 初始化hap 服务器
    let device_manager = IotDeviceManager::new(conn.clone(),
                                               mi_account_manager.clone(),
                                               ble_manager.clone());
    // 初始化iot设备
    device_manager.init().await?;

    let hap_manager = HapManage::new();


    let app_state = AppState::new(conn.clone(), Managers {
        hap_metadata: hap_metadata.clone(),
        hap_manager: hap_manager.clone(),
        device_manager: device_manager.clone(),
        mi_account_manager: mi_account_manager.clone(),
        template_manager: template_manager.clone(),
        ble_manager: ble_manager.clone(),
    });
    // let schema = schema(conn.clone(), None, None)?;



    let app = Router::new()
        // .route("/playground", get(graphql_playground))
        // .route("/graphql", post(graphql_handler))
        .nest_service("/", ServeDir::new("dist/"))
        .nest("/api", router::api())
        .with_state(app_state.clone())
        .layer(socket_io_layer(app_state.clone()));


    let res = APP_CONTEXT.set(ApplicationContext {
        config,
        conn: conn.clone(),
        // js_engine,
        hap_metadata: hap_metadata.clone(),
        device_manager: device_manager.clone(),
        hap_manager: hap_manager.clone(),
    });
    if res.is_err() {
        panic!("APP_CONTEXT 初始化失败");
    }
    let context = get_app_context();

    //初始化集成
    init_integration().await?;
    init_hap_list(&conn, hap_manager.clone(), device_manager.clone()).await?;
    // 初始化hap设备
    // 初始化模板
    template_manager.init().await?;
    info!("api_server start at :{:?}", addr);
    let (api_server_ch_send, api_server_ch) = oneshot::channel::<()>();
    let shutdown_signal = ServerShutdownSignal(Arc::new(Mutex::new(Some(api_server_ch_send))));
    app_state.server_shutdown_signal.lock().await.replace(shutdown_signal.clone());
    let listener = tokio::net::TcpListener::bind(&addr)
        .await?;

    let api_server =
        axum::serve(listener, app.into_make_service())
            .with_graceful_shutdown(async move {
                let _ = api_server_ch.await;
            });

    if let Err(e) = api_server.await {
        error!("api_server error:{:?}", e);
    }
    // context.js_engine.close().await;
    // 服务停止
    drop(app_state);
    device_manager.close().await;
    hap_manager.close().await;
    Ok(())
}

async fn init_integration() -> anyhow::Result<()> {
    let integration = XiaomiIntegration {};
    integration.init()?;
    Ok(())
}

