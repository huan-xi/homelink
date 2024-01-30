use crate::hap::models::{AccessoryModelExt, ReadValueResult};

#[derive(Default)]
pub struct ModelExt;

#[async_trait::async_trait]
impl AccessoryModelExt for ModelExt {
    async fn read_chars_value(&self, cid_tags_list: Vec<&str>) -> ReadValueResult {
        todo!()
    }
}