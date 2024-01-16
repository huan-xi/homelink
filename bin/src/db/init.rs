use std::time::Duration;
use log::info;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
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
pub async fn db_conn() -> DatabaseConnection {
    /*   let link = match CFG.database.link.clone() {
           None => {
               let p = CFG.server.data_dir.as_str();
               match fs::metadata(p) {
                   Ok(f) => {
                       asset!(f.is_dir(), "数据目录不是目录");
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