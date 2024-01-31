use std::sync::Arc;
use anyhow::anyhow;
use dashmap::DashMap;
use once_cell::sync::OnceCell;
use hap::characteristic::{CharReadParam, ReadCharValue, OnReadsFn, OnUpdatesFn, CharUpdateParam, UpdateCharValue};
use miot_spec::device::MiotDevicePointer;
use crate::init::manager::hap_manager::HapManage;

pub mod lumi;
mod deerma;
mod init;

pub static MODEL_EXT_DATABASE: OnceCell<AccessoryModelExtDatabase> = OnceCell::new();


pub struct AccessoryModelExtDatabase {
    pub(crate) model_map: DashMap<String, AccessoryModelExtPointer>,
}

impl AccessoryModelExtDatabase {
    pub fn get(&self, name: &str) -> Option<AccessoryModelExtPointer> {
        self.model_map.get(name).map(|v| v.value().clone())
    }
    pub fn insert(&self, name: &str, ext: AccessoryModelExtPointer) {
        self.model_map.insert(name.to_string(), ext);
    }
}


pub struct AccessoryModelContext {
    pub aid: u64,
    pub(crate) dev: MiotDevicePointer,
    pub(crate) hap_manager: HapManage,
}

pub type AccessoryModelExtPointer = Arc<dyn AccessoryModelExt + Send + Sync + 'static>;
pub type ContextPointer = Arc<AccessoryModelContext>;

/// 配件模型
pub struct AccessoryModel {
    context: Arc<AccessoryModelContext>,
    model_ext: AccessoryModelExtPointer,
}

impl AccessoryModel {
    pub fn new(ctx: AccessoryModelContext, name: &str) -> anyhow::Result<Self> {
        let ext = MODEL_EXT_DATABASE.get_or_init(|| AccessoryModelExtDatabase::default())
            .get(name);
        Ok(Self {
            context: Arc::new(ctx),
            model_ext: ext.ok_or(anyhow!("AccessoryModelExt {} not found",name))?,
        })
    }

    pub fn get_on_reads_fn(&self) -> Option<OnReadsFn> {
        let ext = self.model_ext.clone();
        let ctx = self.context.clone();
        Some(Box::new(move |ids| {
            let ext = ext.clone();
            let ctx = ctx.clone();
            Box::pin(async move {
                let result = ext.read_chars_value(ctx, ids).await?;
                //读取函数
                Ok(result)
            })
        }))
    }

    pub fn get_on_updates_fn(&self) -> Option<OnUpdatesFn> {
        let ext = self.model_ext.clone();
        let ctx = self.context.clone();
        Some(Box::new(move |params| {
            let ext = ext.clone();
            let ctx = ctx.clone();
            Box::pin(async move {
                let result = ext.update_chars_value(ctx, params).await?;
                Ok(result)
            })
        }))
    }
}

pub(crate) type ReadValueResult = anyhow::Result<Vec<ReadCharValue>>;
pub(crate) type UpdateValueResult = anyhow::Result<Vec<UpdateCharValue>>;


/// 配件模型扩展
#[async_trait::async_trait]
pub trait AccessoryModelExt {
    /// 批量读取特征值
    async fn read_chars_value(&self, ctx: ContextPointer, params: Vec<CharReadParam>) -> ReadValueResult;
    async fn update_chars_value(&self, ctx: ContextPointer, params: Vec<CharUpdateParam>) -> UpdateValueResult;
}

