use std::ops::Deref;
use std::sync::{Arc, Mutex};
use dashmap::DashMap;
use sea_orm::DatabaseConnection;
use hap_metadata::metadata::HapMetadata;
use miot_spec::cloud::MiCloud;
use crate::init::manager::device_manager::IotDeviceManager;
use crate::init::manager::hap_manager::HapManage;
use crate::init::manager::mi_account_manager::MiAccountManager;
use crate::init::manager::template_manager::TemplateManager;
use crate::init::Managers;


pub struct AppStateInner {
    conn: DatabaseConnection,
    managers: Managers,
    // hap_server: Arc<IpServer>,
    // pub hap_platform-metadata: Arc<HapMetadata>,
    // pub device_manager: IotDeviceManager,
    // pub hap_manager: HapManage,
    // pub mi_account_manager: MiAccountManager,
    // pub template_manager: TemplateManager,
    //
}

impl AppStateInner {
    pub fn new(conn: DatabaseConnection, managers: Managers,
    ) -> Self {
        let conn_c = conn.clone();
        Self {
            conn,
            managers,
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

impl Deref for AppStateInner {
    type Target = Managers;

    fn deref(&self) -> &Self::Target {
        &self.managers
    }
}

impl AppState {
    pub fn new(conn: DatabaseConnection,
               managers: Managers,
    ) -> Self {
        Self {
            inner: Arc::new(AppStateInner::new(conn, managers)),
        }
    }
}

impl Deref for AppState {
    type Target = Arc<AppStateInner>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}