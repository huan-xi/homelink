

use std::fs::File;
use serde::{Deserialize, Serialize};
use hap_metadata::metadata::{HapMetadata, SystemMetadata};


fn main() {
    let metadata_file = File::open("hap_metadata/gen/system.json").unwrap();

    let metadata: SystemMetadata = serde_json::from_reader(&metadata_file).unwrap();
    let metadata = HapMetadata::from(metadata);
    println!("test");
}
