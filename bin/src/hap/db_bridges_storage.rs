use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use hap::Config;
use hap::pairing::Pairing;
use hap::storage::Storage;
use crate::db::entity::prelude::HapBridgeActiveModel;

pub struct DbBridgesStorage {
    bid: i64,
    conn: DatabaseConnection,
}

#[async_trait]
impl Storage for DbBridgesStorage {
    async fn load_config(&self) -> hap::Result<Config> {
        todo!()
    }

    async fn save_config(&mut self, config: &Config) -> hap::Result<()> {
        let model = HapBridgeActiveModel {
            bridge_id: Default::default(),
            pin_code: Default::default(),
            port: Default::default(),
            category: Default::default(),
            name: Default::default(),
            mac: Default::default(),
            setup_id: Default::default(),
            disabled: Default::default(),
        };
        todo!()
    }

    async fn delete_config(&mut self) -> hap::Result<()> {
        todo!()
    }

    async fn load_aid_cache(&self) -> hap::Result<Vec<u64>> {
        todo!()
    }

    async fn save_aid_cache(&mut self, aid_cache: &[u64]) -> hap::Result<()> {
        todo!()
    }

    async fn delete_aid_cache(&mut self) -> hap::Result<()> {
        todo!()
    }

    async fn load_pairing(&self, id: &uuid::Uuid) -> hap::Result<Pairing> {
        todo!()
    }

    async fn save_pairing(&mut self, pairing: &Pairing) -> hap::Result<()> {
        todo!()
    }

    async fn delete_pairing(&mut self, id: &uuid::Uuid) -> hap::Result<()> {
        todo!()
    }

    async fn list_pairings(&self) -> hap::Result<Vec<Pairing>> {
        todo!()
    }

    async fn count_pairings(&self) -> hap::Result<usize> {
        todo!()
    }

    async fn load_bytes(&self, key: &str) -> hap::Result<Vec<u8>> {
        todo!()
    }

    async fn save_bytes(&mut self, key: &str, value: &[u8]) -> hap::Result<()> {
        todo!()
    }

    async fn delete_bytes(&mut self, key: &str) -> hap::Result<()> {
        todo!()
    }
}