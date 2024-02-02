use log::info;
use sea_orm::Schema;
use sea_orm_migration::prelude::*;
use crate::db::entity::{hap_accessory, hap_bridge, hap_characteristic, hap_service, iot_device, mi_account, miot_device};
use crate::migration::db_utils;
use crate::migration::db_utils::{create_one_table, create_table_index};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        info!("开始创建table----------");
        let db = manager.get_connection();
        let builder = manager.get_database_backend();
        let schema = Schema::new(builder);
        create_one_table(db, builder, &schema, hap_characteristic::Entity).await?;
        // create_table_index(manager, hap_characteristic::Entity, "idx_char_svc_id", vec![hap_characteristic::Column::ServiceId], db_utils::IndexType::Index).await?;
        create_one_table(db, builder, &schema, hap_service::Entity).await?;
        create_one_table(db, builder, &schema, hap_accessory::Entity).await?;
        create_one_table(db, builder, &schema, hap_bridge::Entity).await?;
        //索引
        // create_table_index(manager, hap_bridge::Entity, "udx_pin_code", vec![hap_bridge::Column::PinCode], db_utils::IndexType::Unique).await?;

        create_one_table(db, builder, &schema, iot_device::Entity).await?;
        create_one_table(db, builder, &schema, mi_account::Entity).await?;
        create_one_table(db, builder, &schema, miot_device::Entity).await?;


        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
#[allow(dead_code)]
enum Post {
    Table,
    Id,
    Title,
    Text,
}
