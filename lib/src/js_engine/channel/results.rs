use impl_new::New;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, New)]
#[serde(rename_all = "camelCase")]
pub struct OnCharReadParam {
    // 通道id
    pub error: String,
}
