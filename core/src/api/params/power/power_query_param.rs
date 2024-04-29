use std::collections::HashMap;
use log::info;
use serde_json::Value;
use crate::api::params::power::json_value_to_sea_orm_value;
use sea_orm::*;

// 查询参数
#[derive(serde::Deserialize, Debug)]
pub struct PowerQueryParam {
    page_no: Option<u32>,
    page_size: Option<u32>,
    ///可能是数组[{},{}],{"or":[{}]}
    /// 字符串类型默认like,排除""
    /// 查询过滤器
    filter: Option<String>,
    sort: Option<String>,
}

impl PowerQueryParam {
    pub fn into_query<T>(self) -> anyhow::Result<Select<T>> where
        T: EntityTrait,
        <T as EntityTrait>::Model: Sync, {
        let condition = self.get_condition::<T>()?;
        //处理排序
        let mut select = T::find().filter(condition);
        let order_by = self.get_order_by::<T, T::Column>()?;
        for (column, order) in order_by {
            select = select.order_by(column, order);
        }
        Ok(select)
    }

    pub fn get_order_by<T, C>(&self) -> anyhow::Result<Vec<(C, Order)>> where
        T: EntityTrait<Column=C>,
        C: ColumnTrait,
        <T as EntityTrait>::Model: Sync, {
        let mut order_by = vec![];
        if let Some(sort) = self.sort.as_ref() {
            if !sort.is_empty() {
                let sort = serde_json::from_str::<Value>(sort.as_str())?;
                let sort = sort
                    .as_object()
                    .ok_or(anyhow::anyhow!("sort is not a json object"))?;
                if sort.is_empty() {
                    return Ok(order_by);
                };
                let mut column_map = HashMap::new();
                for c in T::Column::iter() {
                    column_map.insert(c.as_str().to_string(), c);
                }
                for (key, value) in sort.iter() {
                    let column = column_map
                        .get(key.as_str())
                        .ok_or(anyhow::anyhow!("column {} not found", key))?;
                    let order = value.as_str().ok_or(anyhow::anyhow!("order is not a string"))?;

                    let column: C = column.clone();
                    order_by.push((column, match order {
                        "asc" | "ascend" => Order::Asc,
                        "desc" | "descend" => Order::Desc,
                        _ => return Err(anyhow::anyhow!("order is not asc or desc"))
                    }));
                }
            }
        }
        Ok(order_by)
    }


    pub fn get_condition<T>(&self) -> anyhow::Result<Condition> where
        T: EntityTrait,
        <T as EntityTrait>::Model: Sync, {
        let mut condition = Condition::all();
        if let Some(filter) = self.filter.clone() {
            if !filter.is_empty() {
                let filter = serde_json::from_str::<Value>(filter.as_str())?;
                let filter = filter
                    .as_object()
                    .ok_or(anyhow::anyhow!("filter is not a json object"))?;
                if filter.is_empty() {
                    return Ok(condition);
                };
                info!("filter:{:?}", filter);

                let mut column_map = HashMap::new();
                for c in T::Column::iter() {
                    column_map.insert(c.as_str().to_string(), c);
                }
                for (key, value) in filter.iter() {
                    let column = column_map
                        .get(key.as_str())
                        .ok_or(anyhow::anyhow!("column {} not found", key))?;
                    let def = column.def();
                    match value {
                        Value::Object(map) => {}
                        _ => {
                            let value = json_value_to_sea_orm_value::<T>(column.clone(), value.clone())?;
                            match def.get_column_type() {
                                ColumnType::Text | ColumnType::Char(_) | ColumnType::String(_) => {
                                    let str = match value {
                                        sea_orm::Value::String(s) => {
                                            s.map(|s| s.as_ref().clone()).unwrap_or("".to_string())
                                        }
                                        _ => {
                                            return Err(anyhow::anyhow!("column {} value is not string", key));
                                        }
                                    };

                                    if !str.is_empty() {
                                        let value = format!("%{}%", str);
                                        condition = condition.add(column.like(value));
                                    };
                                }
                                _ => {
                                    condition = condition.add(column.eq(value));
                                }
                            }
                        }
                    }
                }
            }
        }


        return Ok(condition);
    }
}