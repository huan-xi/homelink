use std::ops::Deref;
use std::sync::Arc;
use hap::server::IpServer;
use sea_orm::DatabaseConnection;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<AppStateInner>,
}

impl AppState {}

impl Deref for AppState {
    type Target = Arc<AppStateInner>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct AppStateInner {
    conn: DatabaseConnection,
    hap_server: Arc<IpServer>,
}

impl AppStateInner {
    pub fn new(conn: DatabaseConnection, hap_server: Arc<IpServer>) -> Self {
        Self {
            conn,
            hap_server,
        }
    }
    pub fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }
}