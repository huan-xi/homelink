#![allow(unused_variables)]
pub mod hap;


pub mod api;
pub mod config;
pub mod db;
pub mod test;
pub mod init;
pub mod js_engine;
pub mod migration;
mod device;
/// 米家转换模板
pub mod template;

pub type StdDefault = dyn std::default::Default;
