use async_trait::async_trait;
use impl_new::New;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, NotSet};
use tap::TapFallible;
use hap::{Config, Ed25519Keypair};
use hap::Error::Unknown;
use hap::pairing::Pairing;
use hap::storage::Storage;
use crate::db::entity::hap_bridge::{BonjourStatusFlagWrapper, Model};
use crate::db::entity::prelude::{HapBridgeActiveModel, HapBridgeEntity};
use crate::hap::rand_utils::{compute_setup_hash, pin_code_from_str};

#[derive(New)]
pub struct DbBridgesStorage {
    bid: i64,
    conn: DatabaseConnection,
}

#[async_trait]
impl Storage for DbBridgesStorage {
    async fn load_config(&self) -> hap::Result<Config> {
        let bridge = self.find_bridge().await?;
        let pin = pin_code_from_str(bridge.pin_code.to_string().as_str());
        let name = bridge.name.clone();
        let setup_hash = compute_setup_hash(bridge.setup_id.as_str(), bridge.mac.as_str());
        let device_id = bridge.mac.parse()?;
        let bytes = hex::decode(bridge.device_ed25519_keypair.as_str())
            .map_err(|e| Unknown(Box::new(e)))?;
        let device_ed25519_keypair = Ed25519Keypair::from_bytes(bytes.as_slice())
            .map_err(|e| Unknown(Box::new(e)))?;

        Ok(Config {
            port: bridge.port as u16,
            pin,
            name,
            device_id,
            setup_id: bridge.setup_id.clone(),
            setup_hash,
            device_ed25519_keypair,
            configuration_number: bridge.configuration_number as u64,
            state_number: 0,
            category: bridge.category.into(),
            max_peers: bridge.max_peers.map(|i| i as usize),
            status_flag: bridge.status_flag.0,
            ..Default::default()
        })
    }

    async fn save_config(&mut self, config: &Config) -> hap::Result<()> {
        let model = HapBridgeActiveModel {
            bridge_id: Set(self.bid),
            host: Set(Some(config.host.to_string())),
            configuration_number: Set(config.configuration_number as i64),
            status_flag: Set(BonjourStatusFlagWrapper(config.status_flag)),
            create_at: NotSet,
            ..Default::default()
        };
        model.save(&self.conn).await.map_err(|e| Unknown(Box::new(e)))?;
        Ok(())
    }

    async fn delete_config(&mut self) -> hap::Result<()> {
        todo!()
    }

    async fn load_aid_cache(&self) -> hap::Result<Vec<u64>> {
        Ok(vec![])
    }

    async fn save_aid_cache(&mut self, aid_cache: &[u64]) -> hap::Result<()> {
        Ok(())
    }

    async fn delete_aid_cache(&mut self) -> hap::Result<()> {
        Ok(())
    }

    async fn load_pairing(&self, id: &uuid::Uuid) -> hap::Result<Pairing> {
        let p = self.find_bridge().await?
            .pairings.0.remove(id)
            .ok_or(hap::Error::MsgErr("pairing not found"))?;
        Ok(p)
    }

    async fn save_pairing(&mut self, pairing: &Pairing) -> hap::Result<()> {
        let mut bridge = self.find_bridge().await?;
        bridge.pairings.0.insert(pairing.id, pairing.clone());
        let model = HapBridgeActiveModel {
            bridge_id: Set(self.bid),
            pairings: Set(bridge.pairings),
            ..Default::default()
        };
        model.update(&self.conn)
            .await
            .tap_err(|e| println!("save err{:?}", e))
            .map_err(|e| Unknown(Box::new(e)))?;
        Ok(())
    }

    async fn delete_pairing(&mut self, id: &uuid::Uuid) -> hap::Result<()> {
        let mut bridge = self.find_bridge().await?;
        bridge.pairings.0.remove(id);
        let model = HapBridgeActiveModel {
            bridge_id: Set(self.bid),
            pairings: Set(bridge.pairings),
            ..Default::default()
        };
        model.save(&self.conn).await
            .map_err(|e| Unknown(Box::new(e)))?;
        Ok(())
    }

    async fn list_pairings(&self) -> hap::Result<Vec<Pairing>> {
        let bridge = self.find_bridge().await?;
        Ok(bridge.pairings.0.values().cloned().collect())
    }

    async fn count_pairings(&self) -> hap::Result<usize> {
        let bridge = self.find_bridge().await?;
        Ok(bridge.pairings.0.len())
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

impl DbBridgesStorage {
    async fn find_bridge(&self) -> Result<Model, hap::Error> {
        HapBridgeEntity::find_by_id(self.bid)
            .one(&self.conn)
            .await
            .map_err(|e| hap::Error::Unknown(Box::new(e)))?
            .ok_or(hap::Error::MsgErr("bridge not found"))
    }
}

mod test {
    use uuid::Uuid;
    use hap::pairing::{Pairing, Permissions};
    use hap::storage::Storage;
    use crate::db::init::open_db;

    #[tokio::test]
    pub async fn test() {
        let conn = open_db("sqlite:///Users/huanxi/project/homelink/data/data.db".to_string()).await;
        let mut storage = crate::hap::db_bridge_storage::DbBridgesStorage::new(1202261535225806848i64, conn);
        let config = storage.load_config().await.unwrap();
        let pairing = Pairing {
            id: Uuid::parse_str("bc158b86-cabf-432d-aee4-422ef0e3f1d5").unwrap(),
            permissions: Permissions::Admin,
            public_key: [
                215, 90, 152, 1, 130, 177, 10, 183, 213, 75, 254, 211, 201, 100, 7, 58, 14, 225, 114, 243, 218, 166,
                35, 37, 175, 2, 26, 104, 247, 7, 81, 26,
            ],
        };
        storage.save_pairing(&pairing).await.unwrap();

        let a = storage.load_pairing(&pairing.id).await.unwrap();
        println!("{:?}", a);
        storage.delete_pairing(&pairing.id).await.unwrap();

        println!("{:?}", config.status_flag);
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}