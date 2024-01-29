pub mod context;

/// module 通过 channel 通信
#[cfg(feature = "deno")]
pub mod channel;

#[cfg(feature = "deno")]
pub mod ext;
pub mod scripts;
pub mod init_hap_accessory_module;

#[cfg(feature = "deno")]
mod deno_runtime;
#[cfg(feature = "deno")]
pub mod init_js_engine;


#[cfg(not(feature = "deno"))]
pub mod init_js_engine_without_deno;
#[cfg(not(feature = "deno"))]
pub use init_js_engine_without_deno as init_js_engine;
