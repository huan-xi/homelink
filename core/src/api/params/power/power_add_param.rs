use sea_orm::{ActiveModelTrait, ColumnTrait, ColumnTypeTrait, EntityTrait, IdenStatic, IntoActiveModel, Iterable, JsonValue, PrimaryKeyToColumn};
use serde_json::Value;
use crate::api::errors::ApiError;
use crate::api::params::power::json_value_to_sea_orm_value;
use crate::db::entity::iot_device::DeviceType;
use crate::db::enum_factory::get_enum_factory;

#[derive(serde::Deserialize, Debug)]
pub struct PowerAddParam {
    /// json 数据
    pub(crate) data: JsonValue,
}


impl PowerAddParam {
    pub fn to_active_model<T, A>(&self) -> Result<A, ApiError>
        where
            T: EntityTrait,
            <T as EntityTrait>::Model: Sync,
            <T as EntityTrait>::Model: IntoActiveModel<A>,
            A: ActiveModelTrait<Entity=T> + sea_orm::ActiveModelBehavior + std::marker::Send {
        let mut active_model = A::default();
        let mut data = self.data.as_object()
            .ok_or(ApiError::BadRequest("data is not a json object".to_string()))?
            .clone();

        for column in T::Column::iter() {
            //设置字段值
            let column_name = column.as_str();
         /*   if <T::PrimaryKey as PrimaryKeyToColumn>::from_column(column).is_some() {
                //主键
                if !data.contains_key(column_name) {
                    return Err(ApiError::BadRequest(format!("column {} is not in data", column_name)));
                };
            };*/
            if let Some(value) = data.remove(column_name) {
                //value 转换
                let orm_value = json_value_to_sea_orm_value::<T>(column, value)?;
                active_model.set(column, orm_value.into());
            }
        }


        Ok(active_model)
    }
}
