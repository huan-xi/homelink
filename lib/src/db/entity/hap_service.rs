//! `SeaORM` Entity. Generated by sea-orm-hap_platform-metadata 0.11.3

use hap::HapType;
use sea_orm::entity::prelude::*;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};
use crate::hap::hap_type::{MappingHapType};


#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "hap_service"
    }
}


#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Eq, Serialize, Deserialize)]
pub struct Model {
    pub id: i64,
    pub tag: Option<String>,
    /// 配件id
    pub accessory_id: i64,
    pub configured_name: Option<String>,
    pub memo: Option<String>,
    pub service_type: MappingHapType,
    pub disabled: bool,
    pub primary: bool,

}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    AccessoryId,
    ServiceType,
    Memo,
    ConfiguredName,
    Tag,
    Disabled,
    Primary
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    Id,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = i64;
    fn auto_increment() -> bool {
        false
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    HapCharacteristics,
}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::Id => ColumnType::BigInteger.def(),
            Self::AccessoryId => ColumnType::BigInteger.def(),
            Self::ServiceType => ColumnType::Integer.def(),
            Self::Disabled => ColumnType::Boolean.def(),
            Self::Primary => ColumnType::Boolean.def(),
            Self::Memo => ColumnType::String(None).def().null(),
            Self::ConfiguredName => ColumnType::String(Some(64)).def().null(),
            Self::Tag => ColumnType::String(None).def().null(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Relation::HapCharacteristics => {
                Entity::has_many(super::hap_characteristic::Entity).into()
            }
        }
    }
}

impl Related<super::hap_characteristic::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::HapCharacteristics.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}