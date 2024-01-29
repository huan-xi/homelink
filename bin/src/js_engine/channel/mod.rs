pub mod main_channel;
pub mod params;
mod results;

#[cfg(feature = "deno")]
mod resource;

pub type MsgId = u64;

// #[deprecated]
// pub mod mapping_characteristic_channel;
// #[deprecated]
// pub mod hap_channel;