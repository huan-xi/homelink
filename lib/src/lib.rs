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

pub type StdDefault = dyn std::default::Default;