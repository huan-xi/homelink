use std::time::Duration;
use log::info;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use tokio::fs;
use crate::config;
use crate::config::cfgs;
use crate::migration::Migrator;

pub type SeaQuery = sea_orm::sea_query::query::Query;

pub async fn open_db(schema: String) -> DatabaseConnection {
    let mut opt = ConnectOptions::new(schema);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(false);
    let db = Database::connect(opt).await.expect("数据库打开失败");
    info!("Database connected");
    db
}

pub async fn migrator_up(conn: &DatabaseConnection) {
    Migrator::up(conn, None).await.expect("数据库迁移失败");
}

pub async fn db_conn(server_config: &cfgs::Server) -> DatabaseConnection {
    let link = match server_config.db_schema.as_ref() {
        None => {
            let data_dir = server_config.data_dir.as_str();

            match fs::metadata(data_dir).await {
                Ok(f) => {
                    assert!(f.is_dir(), "数据目录不是目录");
                }
                Err(e) => {
                    fs::create_dir_all(data_dir).await.expect(format!("创建数据目录:{:?}失败", data_dir).as_str());
                    info!("设置默认数据,dir:{}", data_dir);
                }
            }
            let path = std::fs::canonicalize(data_dir).expect("初始化数据库失败");
            let schema = format!("sqlite://{}/data.db?mode=rwc", path.to_str().unwrap());

            schema
        }
        Some(str) => str.clone(),
    };
    open_db(link).await
}