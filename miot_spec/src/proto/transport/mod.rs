use futures_util::{FutureExt, SinkExt};

use crate::proto::protocol::RecvMessage;

pub mod udp_iot_spec_proto;
pub mod open_miio_mqtt_proto;
pub mod ble_mapping_proto;
pub mod cloud_miio_proto;
