use async_graphql::{Object, SimpleObject};
use anyhow::Result;
use async_graphql::dynamic::{Field, FieldFuture, FieldValue, Object, Type, TypeRef};

#[derive(SimpleObject)]
struct DemoObject {
    /// Value a
    a: i32,
    /// Value b
    b: i32,
    #[graphql(skip)]
    c: i32,
}

impl DemoObject {
    async fn ab_value(&self) -> i32 {
        self.a + self.b
    }
}
#[derive(SimpleObject)]
pub struct DemoUser {
    pub id: i32,
    pub name: String,
}

pub struct QueryDemo;

impl From<QueryDemo> for Type {
    #[inline]
    fn from(query_demo: QueryDemo) -> Self {
        let query =
            Object::new("QueryDemo").field(
                Field::new("value", TypeRef::named(TypeRef::STRING), |ctx| {
                    FieldFuture::new(async move {
                        Ok(Some(async_graphql::Value::from("abc")))
                    })
                }));

        Type::Object(query)
    }
}
#[Object]
impl QueryDemo {
    async fn user(&self, username: String) -> Result<Option<DemoUser>> {
        Ok(Some(DemoUser {
            id: 1,
            name: username,
        }))
    }
}


pub struct MutationDemo ;
#[Object]
impl MutationDemo {
    async fn signup(&self, username: String, password: String) -> Result<bool> {
        // User signup
        return Ok(true);
    }

    async fn login(&self, username: String, password: String) -> Result<String> {
        // User login (generate token)
        return Ok("token".to_string());
    }
}