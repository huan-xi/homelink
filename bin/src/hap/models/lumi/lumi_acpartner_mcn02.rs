use hap::characteristic::{CharReadParam, CharUpdateParam};
use crate::hap::models::{AccessoryModelExt, ContextPointer, ReadValueResult, UpdateValueResult};


#[derive(Default)]
pub struct ModelExt;

#[async_trait::async_trait]
impl AccessoryModelExt for ModelExt {
    async fn read_chars_value(&self, ctx: ContextPointer, params: Vec<CharReadParam>) -> ReadValueResult {
        todo!()
    }

    async fn update_chars_value(&self, ctx: ContextPointer, params: Vec<CharUpdateParam>) -> UpdateValueResult {
        todo!()
    }
}