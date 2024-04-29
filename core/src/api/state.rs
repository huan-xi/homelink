use std::ops::Deref;
use std::sync::{Arc};
use sea_orm::DatabaseConnection;
use tokio::sync::{Mutex, oneshot, RwLock};
use crate::init::Managers;

#[derive(Default, Clone)]
pub struct ServerShutdownSignal(pub Arc<Mutex<Option<oneshot::Sender<()>>>>);

pub struct ServerShutdownSignal1{

}

impl ServerShutdownSignal {
    pub async fn shutdown(&self) {
        if let Some(sender) = self.0.lock().await.take() {
            let _ = sender.send(());
        } else {
            log::warn!("ServerShutdownSignal::shutdown: already shutdown");
        }
    }
}

pub struct AppStateInner {
    conn: DatabaseConnection,
    managers: Managers,
    pub server_shutdown_signal: Mutex<Option<ServerShutdownSignal>>,

}

impl AppStateInner {
    pub fn new(conn: DatabaseConnection, managers: Managers,
    ) -> Self {
        let conn_c = conn.clone();
        Self {
            conn,
            managers,
            server_shutdown_signal: Default::default(),
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