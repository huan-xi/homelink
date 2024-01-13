use futures_util::future::BoxFuture;
use futures_util::{FutureExt, SinkExt};
use crate::proto::protocol::{JsonMessage, RecvMessage};

pub mod udp_iot_spec_proto;
pub mod open_miio_mqtt_proto;
pub mod ble_mapping_proto;
