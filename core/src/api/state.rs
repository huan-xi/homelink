use std::ops::Deref;
use std::sync::{Arc, Mutex};
use dashmap::DashMap;
use sea_orm::DatabaseConnection;
use crate::init::manager::template_manager::TemplateManager;
use crate::init::Managers;


pub struct AppStateInner {
    conn: DatabaseConnection,
    managers: Managers,

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