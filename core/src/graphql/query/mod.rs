use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};

pub mod demo;



struct Query;

#[Object]
impl Query {
    async fn value(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}


pub fn demo_schema()->Schema<Query, EmptyMutation, EmptySubscription>{
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    schema
}