use std::ops::Deref;
use std::sync::{Arc};
use anyhow::anyhow;
use dashmap::DashMap;
use dashmap::mapref::entry::Entry;
use impl_new::New;
use log::error;
use sea_orm::{DatabaseConnection, EntityTrait};
use sea_orm::ActiveValue::Set;
use tap::TapFallible;
use tokio::sync::RwLock;
use miot_proto::cloud::MiCloud;
use miot_proto::device::cloud_device::MiCloudExt;
use miot_proto::proto::miio_proto::MiotSpecProtocolPointer;
use miot_proto::proto::protocol::ExitError;
use miot_proto::proto::transport::cloud_miio_proto::CloudMiioProto;
use crate::config::context::get_data_dir;
use crate::db::entity::mi_account::MiAccountStatus;
use crate::db::entity::prelude::{MiAccountActiveModel, MiAccountEntity};

/// 米家账号管理器
/// todo 启动任务自动登入
pub struct MiAccountManagerInner {
    pub mi_cloud_map: DashMap<String, Arc<RwLock<MiCloud>>>,
    proto_map: DashMap<String, Arc<CloudMiioProto>>,
    conn: DatabaseConnection,
}


impl MiAccountManagerInner {
    pub async fn add_account(&self, username: String, password: String) -> anyhow::Result<()> {
        let cloud = Self::new_cloud(username.clone(), password).await?;
        self.mi_cloud_map.insert(username, Arc::new(RwLock::new(cloud)));
        Ok(())
    }

    async fn new_cloud(username: String, password: String) -> anyhow::Result<MiCloud> {
        let path = format!("{}/mi_cloud", get_data_dir());
        let cloud = MiCloud::new(path.as_str(), username, Some(password)).await?;
        Ok(cloud)
    }
    /// 获取cloud
    pub async fn get_cloud(&self, account: &str) -> anyhow::Result<Arc<RwLock<MiCloud>>> {
        Ok(match self.mi_cloud_map.entry(account.to_string()) {
            Entry::Occupied(v) => {
                v.get().clone()
            }
            Entry::Vacant(entry) => {
                let model = MiAccountEntity::find_by_id(entry.key())
                    .one(&self.conn)
                    .await?
                    .ok_or(anyhow!("账号:{}不存在",account))?;
                let cloud = Self::new_cloud(model.account, model.password).await?;
                let cloud = Arc::new(RwLock::new(cloud));
                entry.insert(cloud.clone());
                cloud
            }
        })
    }
    pub async fn login(&self, account: &str) -> anyhow::Result<()> {
        let cloud = self.get_cloud(account).await?;
        cloud.write().await.login().await?;
        //将设备状态改成登入
        let account = MiAccountActiveModel {
            account: Set(account.to_string()),
            status: Set(MiAccountStatus::Normal),
            last_login_at: Set(Some(chrono::Utc::now())),
            ..Default::default()
        };
        MiAccountEntity::update(account)
            .exec(&self.conn)
            .await?;
        Ok(())
    }

    /// 获取米家协议给设备
    pub async fn get_proto(&self, account: &str) -> Result<MiotSpecProtocolPointer, ExitError> {
        Ok(match self.proto_map.entry(account.to_string()) {
            Entry::Occupied(v) => {
                v.get().clone()
            }
            Entry::Vacant(entry) => {
                let cloud = self.get_cloud(account).await
                    .tap_err(|e| error!("获取米家云端失败:{}",e))
                    .map_err(|_| ExitError::CloudError)?;
                let proto = CloudMiioProto::new(cloud.clone());
                let proto = Arc::new(proto);
                entry.insert(proto.clone());
                proto
            }
        })
    }
}


#[derive(New)]
pub struct MiCloudDeviceExt {
    account_id: String,
    manager: MiAccountManager,
}

#[async_trait::async_trait]
impl MiCloudExt for MiCloudDeviceExt {
    async fn get_proto(&self) -> Result<MiotSpecProtocolPointer, ExitError> {
        self.manager.get_proto(self.account_id.as_str()).await
    }

    async fn register_property(&self, siid: i32, piid: i32) {}
}


#[derive(Clone)]
pub struct MiAccountManager {
    inner: Arc<MiAccountManagerInner>,
}

impl MiAccountManager {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self {
            inner: Arc::new(MiAccountManagerInner {
                mi_cloud_map: DashMap::new(),
                proto_map: Default::default(),
                conn,
            }),
        }
    }
}

impl Deref for MiAccountManager {
    type Target = Arc<MiAccountManagerInner>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}