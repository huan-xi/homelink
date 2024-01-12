//! `SeaORM` Entity. Generated by sea-orm-hap_metadata 0.11.3

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "hap_bridge"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Eq, Serialize, Deserialize)]
pub struct Model {
    pub bridge_id: i64,
    pub pin_code: i64,
    pub category: i16,
    pub name: String,
    pub disabled: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    BridgeId,
    Name,
    PinCode,
    Category,
    Disabled,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    BridgeId,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = i64;
    fn auto_increment() -> bool {
        false
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::hap_accessory::Entity")]
    HapAccessory,
}

// `Related` trait has to be implemented by hand
impl Related<super::hap_accessory::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::HapAccessory.def()
    }
}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::BridgeId => ColumnType::BigInteger.def(),
            Self::Name => ColumnType::String(None).def(),
            Self::Disabled => ColumnType::Boolean.def(),
            Self::PinCode => ColumnType::BigInteger.def(),
            Self::Category => ColumnType::Integer.def(),
        }
    }
}


impl ActiveModelBehavior for ActiveModel {}
