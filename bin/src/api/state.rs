use std::ops::Deref;
use std::sync::{Arc};
use sea_orm::DatabaseConnection;
use tokio::sync::RwLock;
use hap_metadata::metadata::HapMetadata;
use crate::init::device_manage::IotDeviceManager;
use crate::init::hap_manage::HapManage;


pub struct AppStateInner {
    conn: DatabaseConnection,
    // hap_server: Arc<IpServer>,
    pub hap_metadata: Arc<HapMetadata>,
    pub device_manager: Arc<RwLock<Option<IotDeviceManager>>>,
    pub hap_manager: Arc<RwLock<Option<HapManage>>>,
}

impl AppStateInner {
    pub fn new(conn: DatabaseConnection, hap_metadata: Arc<HapMetadata>) -> Self {
        Self {
            conn,
            hap_metadata,
            device_manager: Arc::new(RwLock::new(None)),
            hap_manager: Arc::new(Default::default()),
        }
    }
    pub async fn hap_manager(&self) -> anyhow::Result<HapManage> {
        self.hap_manager.read().await
            .clone()
            .ok_or(anyhow::anyhow!("hap manager not init"))
    }
    pub fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }
    pub async fn get_iot_device_manager(&self) -> Option<IotDeviceManager> {
        self.device_manager.read().await.clone()
    }
}


#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

impl AppState {
    pub fn new(conn: DatabaseConnection, hap_metadata: Arc<HapMetadata>) -> Self {
        Self {
            inner: Arc::new(AppStateInner::new(conn, hap_metadata)),
        }
    }
}

impl Deref for AppState {
    type Target = Arc<AppStateInner>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}