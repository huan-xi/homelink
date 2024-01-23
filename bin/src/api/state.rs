use std::ops::Deref;
use std::sync::{Arc, Mutex};
use dashmap::DashMap;
use sea_orm::DatabaseConnection;
use tokio::sync::RwLock;
use hap_metadata::metadata::HapMetadata;
use miot_spec::cloud::MiCloud;
use crate::init::device_manage::IotDeviceManager;
use crate::init::hap_manage::HapManage;


pub struct AppStateInner {
    conn: DatabaseConnection,
    // hap_server: Arc<IpServer>,
    pub hap_metadata: Arc<HapMetadata>,
    pub device_manager: IotDeviceManager,
    pub hap_manager: HapManage,
    pub mi_cloud_map: DashMap<String, Arc<MiCloud>>,

}

impl AppStateInner {
    pub fn new(conn: DatabaseConnection,
               hap_metadata: Arc<HapMetadata>,
               device_manager: IotDeviceManager,
               hap_manager: HapManage,
    ) -> Self {
        Self {
            conn,
            hap_metadata,
            device_manager,
            hap_manager,
            mi_cloud_map: Default::default(),
        }
    }

    pub fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }
}


#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

impl AppState {
    pub fn new(conn: DatabaseConnection, hap_metadata: Arc<HapMetadata>,
               device_manager: IotDeviceManager,
               hap_manager: HapManage, ) -> Self {
        Self {
            inner: Arc::new(AppStateInner::new(conn, hap_metadata, device_manager, hap_manager)),
        }
    }
}

impl Deref for AppState {
    type Target = Arc<AppStateInner>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}