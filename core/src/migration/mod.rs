use sea_orm_migration::{MigrationTrait, MigratorTrait};

pub mod m20220101_000001_create_table;
mod db_utils;
mod m20230309_000001_add_column;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20220101_000001_create_table::Migration)]
    }
}