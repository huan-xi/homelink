use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use anyhow::anyhow;
use tap::TapFallible;
use tokio::time::timeout;
use hap::characteristic::delegate::{CharReadParam, CharReadsDelegate, CharUpdateDelegate, CharUpdateParam, ReadCharResults, UpdateCharResults};
use crate::delegate::model::HapModelExtPointer;
use crate::types::CharIdentifier;

#[derive(Clone)]
pub struct ModelDelegates {
    pub delegates: Arc<Vec<ModelDelegate>>,
}

#[async_trait::async_trait]
impl CharReadsDelegate for ModelDelegates {
    fn is_delegate(&self, param: &CharReadParam) -> bool {
        self.delegates.iter().any(|i| CharReadsDelegate::is_delegate(i, param))
    }

    async fn reads_value(&self, params: Vec<CharReadParam>) -> ReadCharResults {
        if self.delegates.len() == 1 {
            return self.delegates.get(0).unwrap().reads_value(params).await;
        }
        todo!("未实现多委托");
    }
}

#[async_trait::async_trait]
impl CharUpdateDelegate for ModelDelegates {
    fn is_delegate(&self, param: &CharUpdateParam) -> bool {
        self.delegates.iter().any(|i| CharUpdateDelegate::is_delegate(i, param))
    }

    async fn on_updates(&self, param: Vec<CharUpdateParam>) -> UpdateCharResults {
        if self.delegates.len() == 1 {
            return self.delegates.get(0).unwrap().on_updates(param).await;
        }
        todo!("未实现多委托");
    }
}


#[derive(Clone)]
pub struct ModelDelegate {
    pub chars: HashSet<CharIdentifier>,
    pub ext: HapModelExtPointer,
    pub timeout: Duration,
}

#[async_trait::async_trait]
impl CharReadsDelegate for ModelDelegate {
    fn is_delegate(&self, param: &CharReadParam) -> bool {
        self.chars.contains(&CharIdentifier::new(param.stag.clone(), param.ctag))
    }

    async fn reads_value(&self, param: Vec<CharReadParam>) -> ReadCharResults {
        //加上超时时间，防止单个模型卡死
        let results = timeout(self.timeout, async {
            self.ext.read_chars_value(param)
                .await
                .tap_err(|e| log::error!("扩展模型读取特征失败:{:?}", e))
        }).await.map_err(|_| anyhow!("委托模型读取超时"))?;
        Ok(results?)
    }
}

#[async_trait::async_trait]
impl CharUpdateDelegate for ModelDelegate {
    fn is_delegate(&self, param: &CharUpdateParam) -> bool {
        self.chars.contains(&CharIdentifier::new(param.stag.clone(), param.ctag))
    }

    async fn on_updates(&self, param: Vec<CharUpdateParam>) -> UpdateCharResults {
        let results = self.ext.update_chars_value(param).await?;
        Ok(results)
    }
}