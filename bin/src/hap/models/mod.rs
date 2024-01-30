use std::sync::Arc;
use anyhow::anyhow;
use dashmap::DashMap;
use log::info;
use once_cell::sync::OnceCell;
use hap::characteristic::leak_detected::Value;
use hap::characteristic::OnReadsFn;

pub mod lumi;

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

impl Default for AccessoryModelExtDatabase {
    fn default() -> Self {
        let model_map: DashMap<String, AccessoryModelExtPointer> = DashMap::new();
        model_map.insert("lumi.acpartner.vmcn02".to_string(), Arc::new(lumi::lumi_acpartner_mcn02::ModelExt::default()));
        model_map.insert("lumi.gateway.mgl03".to_string(), Arc::new(lumi::lumi_gateway_mgl03::ModelExt::default()));
        Self {
            model_map,
        }
    }
}


pub struct AccessoryModelContext {}

pub type AccessoryModelExtPointer = Arc<dyn AccessoryModelExt + Send + Sync + 'static>;

/// 配件模型
pub struct AccessoryModel {
    context: Arc<AccessoryModelContext>,
    model_ext: AccessoryModelExtPointer,
}

impl AccessoryModel {
    pub fn new(ctx: AccessoryModelContext, name: &str) -> anyhow::Result<Self> {
        let ext = MODEL_EXT_DATABASE.get()
            .expect("AccessoryModelExtDatabase 未初始化")
            .get(name);
        Ok(Self {
            context: Arc::new(ctx),
            model_ext: ext.ok_or(anyhow!("AccessoryModelExt {} not found",name))?,
        })
    }
    pub fn on_reads_fn(&self) -> Option<OnReadsFn> {
        Some(Box::new(|ids| {
            Box::pin(async move {
                // let c = self.model_ext.read_chars_value().await?;
                info!("read chars value: {:?}", ids);
                //读取函数
                Ok(vec![])
            })
        }))
    }
}

pub(crate) type ReadValueResult = anyhow::Result<Vec<Option<Value>>>;


/// 配件模型扩展
#[async_trait::async_trait]
pub trait AccessoryModelExt {
    /// 批量读取特征值
    async fn read_chars_value(&self, cid_tags_list: Vec<&str>) -> ReadValueResult;
}

