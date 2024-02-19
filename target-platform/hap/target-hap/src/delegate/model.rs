use std::any::Any;
use std::ops::Deref;
use std::sync::Arc;
use anyhow::anyhow;
use dashmap::DashMap;
use log::error;
use serde_json::Value;
use tap::TapFallible;
use hap::characteristic::delegate::{CharReadParam, CharReadResult, CharReadsDelegate, CharUpdateDelegate, CharUpdateParam, CharUpdateResult};
use hl_integration::event::events::DeviceEvent;
use crate::delegate::database::get_hap_model_ext_database;
use crate::delegate::model_delegates::{ModelDelegate, ModelDelegates};
use crate::hap_manager::HapManage;
use crate::types::{CharIdentifier,  ModelDelegateParam};
use hl_integration::{HlSourceDevice, JsonValue};
pub type HapModelExtPointer = Arc<dyn HapModelExt + Send + Sync + 'static>;
pub type ContextPointer = Arc<AccessoryModelContext>;
pub type AccessoryModelPointer = Arc<HapAccessoryDelegateModel>;

/// 模型扩展上下文
pub struct AccessoryModelContext {
    pub aid: u64,
    pub dev: Arc<dyn HlSourceDevice>,
    pub hap_manager: HapManage,
    /// 资源表,存储局部变量
    pub resource_table: DashMap<String, Box<dyn Any + 'static + Send + Sync>>,
}

impl AccessoryModelContext {
    pub async fn set_char_value(&self, cid: &CharIdentifier, value: Value) {
        // let _ = self.hap_manager.update_char_value(self.aid, cid.stag.clone(), cid.ctag.into(), value).await;
    }
}




/// 配件模型
pub struct HapAccessoryDelegateModel {
    pub model_ext: HapModelExtPointer,

    pub delegate: ModelDelegates,
}

impl HapAccessoryDelegateModel {
    pub async fn new(ctx: AccessoryModelContext, mut delegate_param: Vec<ModelDelegateParam>) -> anyhow::Result<Self> {
        let delegate_param = delegate_param
            .remove(0);

        let name = delegate_param.model.as_str();
        let params = delegate_param.params.clone();

        let ctx = Arc::new(ctx);
        let ext = get_hap_model_ext_database().get(name);
        let model_ext_new_func = ext
            .ok_or(anyhow!("AccessoryModelExt {} not found",name))?;
        let dev = ctx.dev.clone();
        let model_ext = model_ext_new_func(ctx, params)
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

        let delegate = ModelDelegates {
            delegates: Arc::new(vec![ModelDelegate {
                chars: delegate_param.chars.into_iter().map(|i| i.into()).collect(),
                ext: model_ext.clone(),
            }]),
        };

        Ok(Self {
            model_ext,
            delegate,
        })
    }


    pub fn get_on_read_delegate(&self) -> Option<Box<dyn CharReadsDelegate>> {
        let delegate = self.delegate.clone();
        Some(Box::new(delegate))
    }


    pub fn get_on_update_delegate(&self) -> Option<Box<dyn CharUpdateDelegate>> {
        let delegate = self.delegate.clone();
        Some(Box::new(delegate))
    }
}

impl Deref for HapAccessoryDelegateModel {
    type Target = HapModelExtPointer;
    fn deref(&self) -> &Self::Target {
        &self.model_ext
    }
}

pub type ReadValueResult = anyhow::Result<Vec<CharReadResult>>;
pub type UpdateValueResult = anyhow::Result<Vec<CharUpdateResult>>;

pub const PARAM_KEY: &str = "PARAM";


pub trait AccessoryModelExtConstructor {
    /// 创建配件模型扩展
    fn new(ctx: ContextPointer, params: Option<JsonValue>) -> anyhow::Result<HapModelExtPointer>;
}

/// 配件模型扩展
#[async_trait::async_trait]
pub trait HapModelExt {
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
