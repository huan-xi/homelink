use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IdenStatic, IntoActiveModel, JsonValue};
use serde_json::Value;
use crate::db::enum_factory::get_enum_factory;

pub mod power_query_param;
pub mod power_update_param;
pub mod power_add_param;


pub fn json_value_to_sea_orm_value<T>(column: <T as EntityTrait>::Column,
                                     json_value: Value)
                                     -> anyhow::Result<sea_orm::Value>
    where
        T: EntityTrait,
        <T as EntityTrait>::Model: Sync, {
    let value_c = json_value.clone();
    let column_type = column.def().get_column_type().clone();
    match column_type {
        sea_orm::ColumnType::Integer => {
            if let Some(str) = json_value.as_str() {
                //判断是否是枚举类型
                let factory = get_enum_factory();
                if let Some(func) = factory.get_enum(column) {
                    return func(str);
                }
            }

            if let Some(s)=json_value.as_str(){
                let val: i64 = s.parse().map_err(|e| anyhow::anyhow!("column:{},值:{s},转换成i64错误:{e}",column.as_str(),))?;
                return Ok(sea_orm::Value::from(val));
            }

            let val = json_value.as_i64()
                .ok_or(anyhow::anyhow!("Integer 数据转化失败,column:{},json_value:{:?}", column.as_str(), value_c))?;
            // return Ok(sea_orm::Value::from(val as i32));
            todo!();
        }
        sea_orm::ColumnType::Boolean => {
            let val = json_value.as_bool()
                .ok_or(anyhow::anyhow!("数据转化失败,column:{},json_value:{:?}", column.as_str(), value_c))?;
            return Ok(sea_orm::Value::from(val));
        }
        _ => {}
    }

    match json_value {
        Value::Null => {}
        Value::Bool(_) => {}
        Value::Number(num) => {
            if let Some(num) = num.as_i64() {
                return Ok(sea_orm::Value::from(num));
            }
            if let Some(num) = num.as_u64() {
                return Ok(sea_orm::Value::from(num));
            }
            if let Some(num) = num.as_f64() {
                return Ok(sea_orm::Value::from(num));
            }
        }
        Value::String(str) => {
            let def = column.def();

            return match def.get_column_type() {
                sea_orm::ColumnType::BigInteger => {
                    let val: i64 = str.parse().map_err(|e| anyhow::anyhow!("column:{},值:{str},转换成i64错误:{e}",column.as_str(),))?;
                    Ok(sea_orm::Value::from(val))
                }
                _ => {
                    Ok(sea_orm::Value::from(str))
                }
            };
        }
        Value::Array(_) => {}
        Value::Object(obj) => {
            match column.def().get_column_type() {
                sea_orm::ColumnType::Json => {
                    return Ok(sea_orm::Value::Json(Some(Box::new(JsonValue::from(obj)))));
                }
                sea_orm::ColumnType::String(str)=>{
                    return Ok(sea_orm::Value::Json(Some(Box::new(JsonValue::from(obj)))));

                }
                _ => {}
            }
        }
    }
    Err(anyhow::anyhow!("数据转化失败,column:{},columnType:{column_type:?},json_value:{:?}", column.as_str(),value_c))
}