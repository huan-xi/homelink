//! `SeaORM` Entity. Generated by sea-orm-hap_platform-metadata 0.11.3

use sea_orm::entity::prelude::*;
use sea_orm::{FromJsonQueryResult, JsonValue};
use serde::{Deserialize, Serialize};
use target_hap::types::{CharIdentifier, ModelDelegateParam};
use crate::db::entity::common::{Property, PropertyVec};
use crate::db::entity::hap_bridge::BridgeCategory;
use crate::hap::hap_type::MappingHapType;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "hap_accessory"
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult,Default)]
pub struct ModelDelegateParamVec(pub Vec<ModelDelegateParam>);



#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Eq, Serialize, Deserialize)]
pub struct Model {
    pub aid: i64,
    /// 配件名称
    pub name: String,
    pub tag: Option<String>,
    pub device_id: i64,
    pub bridge_id: i64,
    pub disabled: bool,
    pub category: BridgeCategory,
    /*/// 运行js脚本,接管特征的读写事件
    pub script: Option<String>,
    /// 运行js脚本的参数
    pub script_params: Option<String>,*/
    ///hap 配件模型委托 接管读写事件
    pub hap_model_delegates: ModelDelegateParamVec,

    /*    ///配件模型 接管读写事件
        pub model: Option<String>,
        /// 模型 运行时参数
        pub model_params: Option<JsonValue>,*/
    pub memo: Option<String>,
    ///配置信息
    pub info: Option<String>,
    /// 模板id
    pub temp_id: Option<String>,
    pub create_at: chrono::NaiveDateTime,
    pub update_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Aid,
    Tag,
    DeviceId,
    BridgeId,
    Disabled,
    Category,
    Name,
    HapModelDelegates,
    /* Script,
     ScriptParams,
     Model,
     ModelParams,*/
    Memo,
    Info,
    TempId,
    CreateAt,
    UpdateAt,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    Aid,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = i64;
    fn auto_increment() -> bool {
        false
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    HapBridge,
    IotDevice,
}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::DeviceId => ColumnType::BigInteger.def(),
            Self::Tag => ColumnType::String(Some(64)).def().null(),
            Self::Name => ColumnType::String(Some(64)).def(),
            Self::BridgeId => ColumnType::BigInteger.def(),
            Self::Disabled => ColumnType::Boolean.def(),
            Self::Category => ColumnType::Integer.def().null(),
            Self::HapModelDelegates => ColumnType::String(None).def().null(),

            Self::Memo => ColumnType::String(None).def().null(),
            // Self::ModelParams => ColumnType::String(None).def().null(),
            // Self::ScriptParams => ColumnType::String(None).def().null(),
            // Self::Model => ColumnType::String(None).def().null(),
            Self::Info => ColumnType::String(None).def().null(),
            Self::Aid => ColumnType::Integer.def().null(),

            Self::TempId => ColumnType::String(None).def().null(),
            Self::CreateAt => ColumnType::DateTime.def(),
            Self::UpdateAt => ColumnType::DateTime.def(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Relation::HapBridge => {
                Entity::belongs_to(super::hap_bridge::Entity)
                    .from(Column::BridgeId)
                    .to(super::hap_bridge::Column::BridgeId).into()
            }
            Relation::IotDevice => {
                Entity::belongs_to(super::iot_device::Entity)
                    .from(Column::DeviceId)
                    .to(super::iot_device::Column::DeviceId).into()
            }
        }
    }
}

impl Related<super::hap_bridge::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::HapBridge.def()
    }
}

impl Related<super::iot_device::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::IotDevice.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}


#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
    #[sea_orm(entity = "super::iot_device::Entity")]
    IotDevice,

}
