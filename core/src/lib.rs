#![allow(unused_variables)]
pub mod hap;
pub mod api;
pub mod service;
pub mod config;
pub mod db;
pub mod test;
pub mod init;
// pub mod js_engine;
pub mod migration;
mod device;
/// 转换模板
pub mod template;
pub mod unit_convertor;
pub mod socketio;
// pub mod graphql;

pub type StdDefault = dyn std::default::Default;
