use std::net::SocketAddr;
use std::sync::{Arc, mpsc};
use std::time::Duration;
use axum::body::HttpBody;
use axum::Router;
use log::{error, info};
use tokio::sync::oneshot;
use bin::api::router;
use bin::api::state::{AppState, AppStateInner};
use bin::config::cfgs::Configs;
use bin::config::context::{APP_CONTEXT, ApplicationContext, get_app_context};
use bin::db::init::db_conn;
use bin::init;
use bin::init::device_init::init_iot_devices;
use bin::init::device_manage::IotDeviceManager;
use bin::init::hap_manage::HapManage;
use bin::js_engine::ext::env::{EnvContext, EnvContextBuilder};
use bin::js_engine::init_js_engine::init_js_engine;
use hap_metadata::hap_metadata;

pub fn init_context() {}

/// 先创建http服务
/// 创建homekit 服务
/// 创建米家设备
/// 映射米家设备到homekit设备
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    //读取配置
    let config = Configs::init();


    let addr = SocketAddr::new([0, 0, 0, 0].into(), 5514);
    let conn = db_conn().await;
    let hap_metadata = Arc::new(hap_metadata()?);
    // 初始化hap 服务器
    let iot_device_manager = IotDeviceManager::new();
    let hap_manager = HapManage::new();
    let app_state = AppState::new(conn.clone(), hap_metadata.clone(),
                                  iot_device_manager.clone(),
                                  hap_manager.clone());

    let app = Router::new()
        .nest("/api", router::api())
        .with_state(app_state.clone());

    // 初始化js 引擎
    //初始化js 引擎
    let js_engine = init_js_engine(EnvContext {
        info: "home gateway".to_string(),
        version: "v1.0.0.0".to_string(),
        main_channel: None,
        conn: conn.clone(),
        dev_manager: iot_device_manager.clone(),
        hap_manager: hap_manager.clone(),
    })?;
    APP_CONTEXT.set(ApplicationContext {
        config: config.clone(),
        conn,
        js_engine,
        hap_metadata,
        device_manager: iot_device_manager.clone(),
        hap_manager,
    }).unwrap();
    let context = get_app_context();


    let api_server =
        axum::Server::bind(&addr)
            .serve(app.into_make_service());
    info!("api_server start at :{:?}", addr);
    let (mut api_server_ch_send, api_server_ch) = oneshot::channel::<bool>();
    tokio::spawn(async move {
        if let Err(e) = api_server.await {
            error!("api_server error:{:?}", e);
        }
        // 发送到通道
        let _ = api_server_ch_send.send(false);
    });


    // 初始化iot设备
    init_iot_devices(&conn, iot_device_manager.clone()).await?;
    // 初始化hap设备
    init::hap_init::init_hap(&conn, &config, hap_manager.clone(), iot_device_manager.clone()).await?;

    api_server_ch.await?;
    context.js_engine.close()?;

    // 服务停止
    drop(app_state);
    // let _ = js_tx.send(0);
    iot_device_manager.close().await;
    hap_manager.close().await;
    Ok(())
}

#[tokio::test]
pub async fn test() -> anyhow::Result<()> {
    // log
    log4rs::init_file("/Users/huanxi/project/home-gateway/log4rs.yaml", Default::default()).unwrap();
    let conn = db_conn().await;
    let iot_device_manager = init_iot_devices(&conn).await?;

    println!("test1");
    tokio::time::sleep(std::time::Duration::from_secs(100)).await;
    println!("test2");
    drop(iot_device_manager);
    tokio::time::sleep(std::time::Duration::from_secs(100)).await;
    println!("test3");
    Ok(())
}

