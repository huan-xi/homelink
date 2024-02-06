use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::anyhow;
use async_recursion::async_recursion;
use dashmap::DashMap;
use log::error;
use sea_orm::{ActiveModelTrait, DatabaseConnection, TransactionTrait};

use crate::config::context::get_data_dir;
use crate::db::entity::iot_device::SourcePlatform;
use crate::db::entity::prelude::MiotDeviceModel;
use crate::db::service::hap_bridge_service::create_hap_bridge;
use crate::db::SNOWFLAKE;
use crate::hap::template::miot_template::MiotTemplate;
use crate::init::helper::template_helper::{AccessoryCtx, DeviceModelCtx, to_accessory_model, to_char_model, to_device_model, to_service_model};
use crate::init::manager::hap_manager::HapManage;

/// 模板管理器
///



pub struct TemplateManagerInner {
    // platform_templates: DashMap<SourcePlatform, u8>,
    ///model 和模板的映射
    pub mihome_templates: DashMap<String, MiotTemplate>,
}

#[derive(Debug, Clone)]
pub enum SourcePlatformModel {
    ///米家模型
    MiHome(MiotDeviceModel),
}

#[derive(serde::Deserialize, Clone, Copy, Debug)]
pub enum BridgeMode {
    Parent,
    Singer,
}

#[derive( Clone)]
pub struct ApplyTemplateOptions {
    pub(crate) platform: SourcePlatformModel,
    pub(crate) id: String,
    pub hap_manager: HapManage,
    //米家模型
    pub bridge_mode: BridgeMode,
    pub bridge_id: Option<i64>,
    pub gateway_id: Option<i64>,
}

impl TemplateManagerInner {
    /// 模板路径中
    pub async fn init(&self) -> anyhow::Result<()> {
        let dir = get_data_dir();
        let template_dir = PathBuf::from(format!("{dir}/templates"));
        self.scan_dir_start(template_dir).await
    }
    pub async fn scan_dir_start(&self, template_dir: PathBuf) -> anyhow::Result<()> {
        //遍历模板目录
        if template_dir.is_dir() {
            let mut entries = tokio::fs::read_dir(template_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(Ok(plan)) = path.file_name()
                        .and_then(|name| name.to_str())
                        .map(|name| SourcePlatform::from_str(name)) {
                        //遍历目录下的文件
                        self.scan_dir(path).await?;
                    }
                }
            }
        } else {
            error!("模板目录不存在,或不是一个目录:{:?}", template_dir);
        }
        //扫描到模板中
        //对模板校验
        // SourcePlatform::MiHome;

        Ok(())
    }
    #[async_recursion]
    async fn scan_dir(&self, path: PathBuf) -> anyhow::Result<()> {
        let mut files = tokio::fs::read_dir(path).await?;
        while let Some(file) = files.next_entry().await? {
            let file_path = file.path();
            if file_path.is_file() {
                if Some(true) == file_path.extension()
                    .map(|i| i == "toml") {
                    //读取文件内容
                    let content = tokio::fs::read_to_string(file_path).await?;
                    //解析文件内容
                    match MiotTemplate::from_str(content.as_str()) {
                        Ok(temp) => {
                            self.mihome_templates.insert(temp.id.clone(), temp);
                        }
                        Err(e) => {
                            log::error!("解析模板文件:{:?}\n失败:{:?}", file.path(),e);
                        }
                    }
                }
            } else {
                //递归
                self.scan_dir(file_path).await?;
            }
        }
        Ok(())
    }

    pub fn has_template(&self, platform: SourcePlatform, model: &str) -> bool {
        match platform {
            SourcePlatform::MiHome => {
                if self.mihome_templates.contains_key(model) {
                    return true;
                };
                for t in self.mihome_templates.iter() {
                    if t.model.as_str() == model {
                        return true;
                    };
                }
            }
        }
        false
    }

    /// 应用模板
    pub async fn apply_template(&self, conn: &DatabaseConnection, options: &ApplyTemplateOptions) -> anyhow::Result<()> {
        // self.mihome_templates.get(model).map(|v| v.clone())

        match &options.platform {
            SourcePlatformModel::MiHome(model) => {
                self.apply_mihome_template(conn, model, &options).await?;
            }
        }
        Ok(())
    }
    /// 应用米家模板
    pub async fn apply_mihome_template(&self, conn: &DatabaseConnection, model: &MiotDeviceModel, option: &ApplyTemplateOptions) -> anyhow::Result<()> {
        let temp = self.mihome_templates.get(option.id.as_str())
            .ok_or(anyhow!("模板不存在"))?
            .clone();
        //开启事务
        let txn = conn.begin().await?;

        for device in temp.devices.iter() {
            let device_id = SNOWFLAKE.next_id();
            let name = device.display_name.clone().unwrap_or(model.name.clone());
            let dev_ctx = DeviceModelCtx {
                device_id,
                name: name.clone(),
                id: temp.id.clone(),
                did: model.did.clone(),
                version: temp.version.clone(),
            };
            let dev_model = to_device_model(dev_ctx.clone(), device)?;
            dev_model.insert(&txn).await?;
            //创建配件
            for accessory in device.accessories.iter() {
                let aid = SNOWFLAKE.next_id();
                //桥接器
                let bridge_id = match option.bridge_mode {
                    BridgeMode::Parent => {
                        option.bridge_id
                            .ok_or(anyhow!("未设置桥接器"))?
                    }
                    BridgeMode::Singer => {
                        //创建桥接器
                        let model = create_hap_bridge(&txn, None, accessory.category, name.clone(), true).await?;
                        model.bridge_id
                    }
                };
                let ctx = AccessoryCtx {
                    aid,
                    bridge_id,
                    dev_ctx: dev_ctx.clone(),
                };
                let accessory_model = to_accessory_model(ctx, accessory)?;
                accessory_model.insert(&txn).await?;
                for service in accessory.services.iter() {
                    let sid = SNOWFLAKE.next_id();
                    // 保存服务
                    to_service_model(aid, sid, service)?.insert(&txn).await?;
                    for char in service.characteristics.iter() {
                        let default = option.hap_manager.get_hap_default_info(char.char_type.into())
                            .ok_or(anyhow!("未找到hap默认信息"))?;
                        let char_model = to_char_model(sid, char, default)?;
                        char_model.insert(&txn).await?;
                    }
                }
                //bridge_id
            }
        }
        txn.commit().await?;

        //转设备


        Ok(())
    }
}


#[derive(Clone)]
pub struct TemplateManager {
    inner: Arc<TemplateManagerInner>,
}

impl Deref for TemplateManager {
    type Target = TemplateManagerInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TemplateManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(TemplateManagerInner {
                mihome_templates: Default::default(),
            }),
        }
    }
}

mod test {
    use std::path::PathBuf;

    use crate::db::entity::prelude::HapAccessoryActiveModel;
    use crate::init::manager::template_manager::TemplateManager;

    #[tokio::test]
    pub async fn test() {
        let manager = TemplateManager::new();
        let template_dir = PathBuf::from("/Users/huanxi/project/homelink/data/templates");
        let a = manager.scan_dir_start(template_dir).await.unwrap();
        for a in &manager.mihome_templates {
            println!("{:?}", a.key());
        }
        // println!("{:?}", manager.mihome_templates);
        //转hap
        /*let temp = manager.mihome_templates.get("").unwrap().clone();
        for device in temp.devices.into_iter() {
            for accessory in device.accessories {}
        };*/

        /* HapAccessoryActiveModel {
             aid: Default::default(),
             name: Default::default(),
             tag: Default::default(),
             device_id: Default::default(),
             bridge_id: Default::default(),
             disabled: Default::default(),
             category: Default::default(),
             script: Default::default(),
             model: Default::default(),
             params: Default::default(),
             memo: Default::default(),
             info: Default::default(),
             temp_id: Default::default(),
             create_at: Default::default(),
             update_at: Default::default(),
         };*/


        //cesgu
    }
}
