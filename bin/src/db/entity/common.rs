use impl_new::New;
use sea_orm::{ColumnType, FromJsonQueryResult, Value};
use sea_orm::sea_query::{ArrayType, ValueTypeErr};
use serde::{Deserialize, Serialize};
use serde_aux::prelude::deserialize_number_from_string;
#[derive(Clone, Debug, PartialEq, Eq, Default, Deserialize, Serialize, FromJsonQueryResult)]
pub struct PropertyVec(pub Vec<Property>);
#[derive(Clone, Debug, Copy, PartialEq, Eq, Default, Deserialize, Serialize, FromJsonQueryResult,New)]
pub struct Property {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub siid: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub piid: i32,
    //单位
}
