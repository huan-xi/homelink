pub mod hap;
pub mod api;
pub mod config;
pub mod db;
pub mod test;
pub mod init;
mod service;
mod iot_device;

pub type StdDefault = dyn std::default::Default;