use rust_embed::RustEmbed;
#[derive(RustEmbed)]
#[folder = "src/js_engine/scripts/asset"]
pub struct ScriptAsset;