use crate::db::snowflake::MySnowflakeGenerator;
use once_cell::sync::Lazy;
pub mod entity;
pub mod service;
pub mod snowflake;
pub mod init;

pub static SNOWFLAKE: Lazy<MySnowflakeGenerator> = Lazy::new(MySnowflakeGenerator::default);
