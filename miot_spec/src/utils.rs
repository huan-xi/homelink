use std::time::{SystemTime, UNIX_EPOCH};

pub fn timestamp() -> u64 {
    let now = SystemTime::now();
    now.duration_since(UNIX_EPOCH).unwrap().as_secs()
}