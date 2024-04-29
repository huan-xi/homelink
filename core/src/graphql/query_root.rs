use crate::db::entity::*;
use async_graphql::dynamic::*;
use async_graphql::{InputType, Scalar, ScalarType};
use sea_orm::DatabaseConnection;
use seaography::{Builder, BuilderContext};

lazy_static::lazy_static! { static ref CONTEXT : BuilderContext = BuilderContext :: default () ; }


struct StringNumber(i64);


pub fn schema(
    database: DatabaseConnection,
    depth: Option<usize>,
    complexity: Option<usize>,
) -> Result<Schema, SchemaError> {
    let mut builder = Builder::new(&CONTEXT, database.clone());

    seaography::register_entities!(
        builder,
        [
            hap_accessory,
            iot_device,
            miot_device,
            mi_account,
            hap_bridge,
            hap_service,
            hap_characteristic,
        ]
    );

    let schema = builder.schema_builder();
    let schema = if let Some(depth) = depth {
        schema.limit_depth(depth)
    } else {
        schema
    };
    let schema = if let Some(complexity) = complexity {
        schema.limit_complexity(complexity)
    } else {
        schema
    };
    // let scalar = Scalar::new("StringNumber");
    // scalar.convert_to(|v| {
    //     Ok(v.clone())
    // });

    // let schema = schema.register(Scalar::new(StringNumber::type_name()));


    schema.data(database).finish()
}
