use std::borrow::Cow;
use deno_runtime::deno_core::Resource;
use crate::js_engine::channel::main_channel::ReceiverResource;


impl Resource for ReceiverResource {
    fn name(&self) -> Cow<str> {
        "ReceiverResource".into()
    }
}