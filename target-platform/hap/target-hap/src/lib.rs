pub mod hap_manager;
pub mod delegate;

pub mod types;
/// 模型的特殊处理
// pub mod models;
pub mod iot;
pub mod hap_type_wrapper;


use std::sync::Arc;
use tokio::sync::RwLock;
use hap::accessory::HapAccessory;

pub type HapAccessoryPointer = Arc<RwLock<Box<dyn HapAccessory>>>;

//re export
pub use hap;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        println!("it_works");
    }
}
