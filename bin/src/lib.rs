pub mod hap;
pub mod api;
pub mod config;
pub mod db;
pub mod test;
pub mod init;
mod service;
mod iot_device;
pub mod js_engine;

pub type StdDefault = dyn std::default::Default;