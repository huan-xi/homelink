use std::env;
use std::env::VarError;
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::anyhow;
use async_recursion::async_recursion;
use axum::http::Method;
use dashmap::DashMap;
use log::error;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, IntoActiveModel, NotSet, QueryFilter, TransactionTrait, TryIntoModel};
use sea_orm::ActiveValue::Set;
use target_hap::hap_manager::HapManage;

use crate::config::context::get_data_dir;
use crate::db::entity::hap_accessory::ModelDelegateParamVec;
use crate::db::entity::hap_characteristic::HapCharInfoQueryResult;
use crate::db::entity::iot_device::{ActiveModel, Model, SourcePlatform};
use crate::db::entity::prelude::{HapAccessoryActiveModel, HapAccessoryColumn, HapAccessoryEntity, HapCharacteristicActiveModel, HapCharacteristicColumn, HapCharacteristicEntity, HapServiceActiveModel, HapServiceColumn, HapServiceEntity, IotDeviceColumn, IotDeviceEntity, MiotDeviceModel};
use crate::db::service::hap_bridge_service::create_hap_bridge;
use crate::db::SNOWFLAKE;
use crate::init::helper::template_helper::{AccessoryCtx, DeviceModelCtx, to_accessory_model, to_char_model, to_device_model, to_service_model};
use crate::template::hap::accessory::AccessoryTemplate;
use crate::template::hap::service::ServiceTemplate;
use crate::template::hl_template::{HlDeviceTemplate};

/// 模板管理器
///

pub struct TemplateManagerInner {
    // platform_templates: DashMap<SourcePlatform, u8>,
    ///model 和模板的映射
    pub templates: DashMap<String, HlDeviceTemplate>,
}

#[derive(Debug, Clone)]
pub enum SourcePlatformModel {
    ///米家模型
    MiHome(MiotDeviceModel),
}

#[derive(Debug, Copy, Clone)]
pub enum ApplyMethod {
    Update,
    Create,
}

#[derive(serde::Deserialize, Clone, Copy, Debug)]
pub enum BridgeMode {
    Parent,
    Singer,
}

#[derive(Clone)]
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
        let template_dir = match env::var("TEMPLATES_DIR") {
            Ok(v) => {
                PathBuf::from(v.as_str())
            }
            Err(_) => {
                PathBuf::from(format!("{dir}/templates"))
            }
        };
        self.scan_dir_start(template_dir).await
    }
    pub async fn scan_dir_start(&self, template_dir: PathBuf) -> anyhow::Result<()> {
        //遍历模板目录
        if template_dir.is_dir() {
            let mut entries = tokio::fs::read_dir(template_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_dir() {
                    self.scan_dir(path).await?;
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
                    match HlDeviceTemplate::from_str(content.as_str()) {
                        Ok(temp) => {
                            self.templates.insert(temp.id.clone(), temp);
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
            SourcePlatform::Mijia => {
                if self.templates.contains_key(model) {
                    return true;
                };
                for t in self.templates.iter() {
                    if t.model.as_str() == model {
                        return true;
                    };
                }
            }
            _ => {
                return false;
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
        let temp = self.templates
            .get(option.id.as_str())
            .ok_or(anyhow!("模板不存在"))?
            .clone();
        //开启事务
        let txn = conn.begin().await?;
        let batch_id = SNOWFLAKE.next_id();
        for device in temp.devices.iter() {
            let device_id = SNOWFLAKE.next_id();
            let name = device.display_name.clone().unwrap_or(model.name.clone());
            let mut dev_ctx = DeviceModelCtx {
                device_id,
                name: name.clone(),
                id: temp.id.clone(),
                did: model.did.clone(),
                version: temp.version.clone(),
                temp_batch_id: batch_id,
            };
            let dev_model = to_device_model(dev_ctx.clone(), device)?;
            //判断是否更新
            dev_ctx.device_id = save_or_update_device_model(&txn, temp.id.as_str(), dev_model).await?;

            //创建配件
            for accessory in device.accessories.iter() {
                let mut aid = SNOWFLAKE.next_id();
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
                //save_or_update
                aid = save_or_update_accessory(&txn, accessory_model).await?;

                for service in accessory.services.iter() {
                    let mut sid = SNOWFLAKE.next_id();
                    // 保存服务
                    let model = to_service_model(aid, sid, service)?;
                    sid = save_or_update_service(&txn, model).await?;
                    //.insert(&txn).await?;
                    for char in service.chars.iter() {
                        let default = option.hap_manager.get_hap_default_info(char.char_type.into())
                            .ok_or(anyhow!("未找到hap默认信息"))?;
                        let char_model = to_char_model(sid, char, default)?;
                        save_or_update_char(&txn, char_model).await?;
                    }
                }
                //bridge_id
            }
        }
        txn.commit().await?;

        //转设备


        Ok(())
    }


    pub async fn update_accessory(&self,
                                  conn: &DatabaseConnection,
                                  accessory: AccessoryTemplate) -> anyhow::Result<()> {
        let accessory_model = accessory.clone().try_into_update_model()?;
        let txn = conn.begin().await?;
        accessory_model.update(&txn).await?;
        //服务转换
        for svc in accessory.services.into_iter() {
            for char in svc.chars.clone().into_iter() {
                let char_model = char.try_into_update_model()?;
                char_model.update(&txn).await?;
            }
            let service_model = svc.try_into_update_model()?;
            service_model.update(&txn).await?;
        }

        txn.commit().await?;
        Ok(())
    }


}

//ActiveModelTrait
async fn save_or_update<T: ActiveModelTrait + Send + sea_orm::ActiveModelBehavior>(method: ApplyMethod, txn: &DatabaseTransaction, model: T) -> anyhow::Result<()>
    where <<T as sea_orm::ActiveModelTrait>::Entity as sea_orm::EntityTrait>::Model: IntoActiveModel<T> {
    match method {
        ApplyMethod::Update => {
            model.update(txn).await?;
        }
        ApplyMethod::Create => {
            model.insert(txn).await?;
        }
    }
    Ok(())
}

async fn save_or_update_char(txn: &DatabaseTransaction, mut model: HapCharacteristicActiveModel) -> anyhow::Result<i64> {
    let old = HapCharacteristicEntity::find()
        .filter(HapCharacteristicColumn::ServiceId.eq(model.service_id.clone().unwrap())
            .and(HapCharacteristicColumn::CharacteristicType.eq(model.characteristic_type.clone().unwrap())))
        .one(txn).await?;
    match old {
        None => {
            let cid = model.cid.clone().unwrap();
            model.insert(txn).await?;
            Ok(cid)
        }
        Some(old) => {
            model.cid = Set(old.cid);
            HapCharacteristicEntity::update(model).exec(txn).await?;
            Ok(old.cid)
        }
    }
}

async fn save_or_update_service(txn: &DatabaseTransaction, mut model: HapServiceActiveModel) -> anyhow::Result<i64> {
    let old = HapServiceEntity::find()
        .filter(HapServiceColumn::Tag.eq(model.tag.clone().unwrap())
            .and(HapServiceColumn::AccessoryId.eq(model.accessory_id.clone().unwrap())))
        .one(txn).await?;
    match old {
        None => {
            let sid = model.id.clone().unwrap();
            model.insert(txn).await?;
            Ok(sid)
        }
        Some(old) => {
            model.id = Set(old.id);
            HapServiceEntity::update(model).exec(txn).await?;
            Ok(old.id)
        }
    }
}

async fn save_or_update_accessory(txn: &DatabaseTransaction, mut model: HapAccessoryActiveModel) -> anyhow::Result<i64> {
    let old = HapAccessoryEntity::find()
        .filter(HapAccessoryColumn::DeviceId.eq(model.device_id.clone().unwrap())
            .and(HapAccessoryColumn::Tag.eq(model.tag.clone().unwrap()))
            .and(HapAccessoryColumn::TempId.eq(model.temp_id.clone().unwrap())))
        .one(txn)
        .await?;
    match old {
        None => {
            let aid = model.aid.clone().unwrap();
            model.insert(txn).await?;
            Ok(aid)
        }
        Some(old) => {
            model.aid = Set(old.aid);
            // model.temp_id = NotSet;
            HapAccessoryEntity::update(model).exec(txn).await?;
            Ok(old.aid)
        }
    }
}


/// 模板转模型
pub fn into_accessory_model(method: ApplyMethod, temp: &AccessoryTemplate) -> anyhow::Result<HapAccessoryActiveModel> {
    let now = Set(chrono::Local::now().naive_local());
    let mut delegate = vec![];
    for t in temp.hap_delegates.clone().into_iter() {
        delegate.push(t.try_into()?);
    }

    Ok(HapAccessoryActiveModel {
        aid: Set(match method {
            ApplyMethod::Update => temp.aid.ok_or(anyhow!("aid不能为空"))?,
            ApplyMethod::Create => SNOWFLAKE.next_id(),
        }),
        name: Set(temp.name.clone().ok_or(anyhow!("name不能为空"))?),
        tag: Set(Some(temp.tag.clone())),
        device_id: Set(temp.device_id.ok_or(anyhow!("device_id不能为空"))?),
        bridge_id: Set(temp.bridge_id.ok_or(anyhow!("bridge_id不能为空"))?),
        disabled: Set(temp.disabled.unwrap_or(false)),
        category: Set(temp.category),
        hap_model_delegates: Set(ModelDelegateParamVec(delegate)),
        memo: Set(temp.memo.clone()),
        info: Default::default(),
        temp_id: Default::default(),
        create_at: match method {
            ApplyMethod::Create => now.clone(),
            ApplyMethod::Update => NotSet
        },
        update_at: now,
    })
}

async fn save_or_update_device_model(txn: &DatabaseTransaction,
                                     temp_id: &str,
                                     mut dev_model: ActiveModel) -> anyhow::Result<i64> {
    let old = IotDeviceEntity::find()
        .filter(
            IotDeviceColumn::TempId.eq(temp_id.to_string())
                .and(IotDeviceColumn::SourcePlatform.eq(dev_model.source_platform.clone().unwrap()))
                .and(IotDeviceColumn::SourceId.eq(dev_model.source_id.clone().unwrap()))
                .and(IotDeviceColumn::Tag.eq(dev_model.tag.clone().unwrap()))
        )
        .one(txn)
        .await?;
    Ok(match old {
        None => {
            let did = dev_model.device_id.clone().unwrap();
            dev_model.insert(txn).await?;
            did
        }
        Some(old) => {
            dev_model.device_id = Set(old.device_id);


            dev_model.temp_batch_id = NotSet;
            IotDeviceEntity::update(dev_model).exec(txn).await?;
            old.device_id
        }
    })
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
                templates: Default::default(),
            }),
        }
    }
}

mod test {
    use std::path::PathBuf;
    use std::str::FromStr;

    use crate::db::entity::prelude::HapAccessoryActiveModel;
    use crate::init::logger_init::init_logger;
    use crate::init::manager::template_manager::TemplateManager;
    use crate::template::hl_template::HlDeviceTemplate;


    #[tokio::test]
    pub async fn test_file() {
        init_logger();
        let template_dir = PathBuf::from("/Users/huanxi/project/homelink/data/templates/mijia/yeelink/yeelink.light.lamp22.toml");

        let content = tokio::fs::read_to_string(template_dir).await.unwrap();
        //解析文件内容
        match HlDeviceTemplate::from_str(content.as_str()) {
            Ok(temp) => {
                log::info!("{:?}", temp);
                //self.mihome_templates.insert(temp.id.clone(), temp);
            }
            Err(e) => {
                log::error!("解析模板文件:\n失败:{:?}", e);
            }
        }
    }

    #[tokio::test]
    pub async fn test() {
        init_logger();
        let manager = TemplateManager::new();
        let template_dir = PathBuf::from("/Users/huanxi/project/homelink/data/templates");
        let a = manager.scan_dir_start(template_dir).await.unwrap();
        for a in &manager.templates {
            println!("{:?}", a.key());
        }


        //cesgu
    }
}

