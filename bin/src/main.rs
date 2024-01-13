use std::net::SocketAddr;
use std::sync::{Arc, mpsc};
use std::time::Duration;
use axum::body::HttpBody;
use axum::Router;
use log::{error, info};
use sea_orm::{ColumnTrait, ConnectOptions, Database, DatabaseConnection, EntityTrait, QueryFilter};
use bin::api::router;
use bin::api::state::{AppState, AppStateInner};
use bin::config::cfgs::Configs;
use bin::init;
use bin::init::device_init::init_iot_devices;
use hap_metadata::hap_metadata;
use miot_spec::device::miot_spec_device::{DeviceInfo, MiotSpecDevice};


pub async fn db_conn() -> DatabaseConnection {
    /*   let link = match CFG.database.link.clone() {
           None => {
               let p = CFG.server.data_dir.as_str();
               match fs::metadata(p) {
                   Ok(f) => {
                       assert!(f.is_dir(), "数据目录不是目录");
                   }
                   Err(e) => {
                       fs::create_dir_all(p).expect(format!("创建数据目录:{:?}失败", p).as_str());
                   }
               }
               let path = std::fs::canonicalize(CFG.server.data_dir.as_str()).unwrap();
               let schema = format!("sqlite://{}/data.db?mode=rwc", path.to_str().unwrap());
               println!("设置默认数据:{}", schema);
               schema
           }
           Some(str) => str.clone(),
       };

       open_db(link).await*/
    let path = std::fs::canonicalize("/Users/huanxi/project/home-gateway/data").unwrap();
    let schema = format!("sqlite://{}/data.db?mode=rwc", path.to_str().unwrap());
    open_db(schema).await
}

pub async fn open_db(schema: String) -> DatabaseConnection {
    let mut opt = ConnectOptions::new(schema);
    opt.max_connections(1000)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(false);
    let db = Database::connect(opt).await.expect("数据库打开失败");
    info!("Database connected");
    db
}

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
    let app_state = AppState::new(conn.clone(), hap_metadata.clone());

    let app = Router::new()
        .nest("/api", router::api())
        .with_state(app_state.clone());

    let api_server =
        axum_server::Server::bind(addr.clone())
            .serve(app.into_make_service());
    info!("api_server start at :{:?}", addr);
    let (mut api_server_ch_send, api_server_ch) = tokio::sync::oneshot::channel::<bool>();
    tokio::spawn(async move {
        if let Err(e) = api_server.await {
            error!("api_server error:{:?}", e);
        }
        // 发送到通道
        let _ = api_server_ch_send.send(false);
    });

    // 初始化iot设备
    let iot_device_manager = init_iot_devices(&conn).await?;
    app_state.device_manager.write().await.replace(iot_device_manager.clone());
    // 初始化hap设备
    let hap_manager = init::hap_init::init_hap(&conn, &config, iot_device_manager.clone()).await?;
    app_state.hap_manager.write().await.replace(hap_manager.clone());
    api_server_ch.await?;
    // 服务停止
    drop(app_state);
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

