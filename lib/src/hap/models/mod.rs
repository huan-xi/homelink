use std::any::Any;
use std::ops::Deref;
use std::sync::Arc;
use anyhow::anyhow;
use dashmap::DashMap;
use log::{error, info};
use once_cell::sync::OnceCell;
use sea_orm::JsonValue;
use serde_json::Value;
use tap::TapFallible;
use hap::characteristic::{CharReadParam, CharReadResult, OnReadsFn, OnUpdatesFn, CharUpdateParam, CharUpdateResult};
use hl_device::event::events::DeviceEvent;
use miot_spec::device::common::emitter::EventType;
use miot_spec::device::MiotDevicePointer;
use crate::hap::models::model_ext_database::{AccessoryModelExtDatabase, MODEL_EXT_DATABASE};
use crate::init::DevicePointer;
use crate::init::manager::hap_manager::HapManage;

pub mod lumi;
mod deerma;
mod model_ext_database;
mod common;


pub struct AccessoryModelContext {
    pub aid: u64,
    pub(crate) dev: DevicePointer,
    pub(crate) hap_manager: HapManage,
    /// 资源表,存储局部变量
    pub resource_table: DashMap<String, Box<dyn Any + 'static + Send + Sync>>,
}

pub type AccessoryModelExtPointer = Arc<dyn AccessoryModelExt + Send + Sync + 'static>;

pub type ContextPointer = Arc<AccessoryModelContext>;
pub type AccessoryModelPointer = Arc<AccessoryModel>;

/// 配件模型
#[derive(Clone)]
pub struct AccessoryModel {
    pub model_ext: AccessoryModelExtPointer,
}

impl AccessoryModel {
    pub async fn new(ctx: AccessoryModelContext, name: &str, option: Option<Value>) -> anyhow::Result<Self> {
        let ctx = Arc::new(ctx);
        let ext = MODEL_EXT_DATABASE.get_or_init(|| AccessoryModelExtDatabase::default()).get(name);
        let model_ext_new_func = ext
            .ok_or(anyhow!("AccessoryModelExt {} not found",name))?;
        let dev = ctx.dev.clone();
        let model_ext = model_ext_new_func(ctx, option)
            .tap_err(|e| error!("创建模型扩展失败{:?}",e))?;
        //订阅设备事件
        if model_ext.is_subscribe_event() {
            let model_ext_c = model_ext.clone();
            dev.add_listener(Box::new(move |data| {
                let model_ext = model_ext_c.clone();
                Box::pin(async move {
                    model_ext.on_event(data).await;
                    ()
                })
            })
            ).await;
        }

        Ok(Self {
            model_ext,
        })
    }


    pub fn get_on_reads_fn(&self) -> Option<OnReadsFn> {
        let ext = self.model_ext.clone();
        Some(Box::new(move |ids| {
            let ext = ext.clone();
            Box::pin(async move {
                let result = ext.read_chars_value(ids).await?;
                //读取函数
                Ok(result)
            })
        }))
    }


    pub fn get_on_updates_fn(&self) -> Option<OnUpdatesFn> {
        let ext = self.model_ext.clone();
        Some(Box::new(move |params| {
            let ext = ext.clone();
            Box::pin(async move {
                let result = ext.update_chars_value(params).await?;
                Ok(result)
            })
        }))
    }
}

impl Deref for AccessoryModel {
    type Target = AccessoryModelExtPointer;
    fn deref(&self) -> &Self::Target {
        &self.model_ext
    }
}

pub(crate) type ReadValueResult = anyhow::Result<Vec<CharReadResult>>;
pub(crate) type UpdateValueResult = anyhow::Result<Vec<CharUpdateResult>>;

pub(crate) const PARAM_KEY: &str = "PARAM";


pub trait AccessoryModelExtConstructor {
    /// 创建配件模型扩展
    fn new(ctx: ContextPointer, params: Option<JsonValue>) -> anyhow::Result<AccessoryModelExtPointer>;
}

/// 配件模型扩展
#[async_trait::async_trait]
pub trait AccessoryModelExt {
    /// 初始化
    async fn init(&self) -> anyhow::Result<()> {
        Ok(())
    }
    /// 批量读取特征值
    async fn read_chars_value(&self, params: Vec<CharReadParam>) -> ReadValueResult;
    async fn update_chars_value(&self, params: Vec<CharUpdateParam>) -> UpdateValueResult;
    async fn on_event(&self, event_type: DeviceEvent) {}

    /// 是否订阅设备的事件
    fn is_subscribe_event(&self) -> bool { true }
}
