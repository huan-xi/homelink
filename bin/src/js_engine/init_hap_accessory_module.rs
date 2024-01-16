use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use deno_runtime::deno_core::url::Url;
use deno_runtime::deno_core::v8::String;
use hex::ToHex;
use log::error;
use tap::TapFallible;
use crate::config::context::get_app_context;
use crate::init::hap_manage::HapManage;
use crate::js_engine::channel::hap_channel;
use crate::js_engine::channel::hap_channel::ToHapModuleSender;
use crate::js_engine::init_js_engine::{EngineEvent, ExecuteSideModuleParam};

/// hap 设备作为一个module 运行

pub async fn init_hap_accessory_module(manage: HapManage, aid: i64, script: &str) -> anyhow::Result<Arc<ToHapModuleSender>> {
    let (sender, recv, exit_ch) = hap_channel::channel();
    //注册到context
    let url = init_js_files(aid, script)?;
    //注册到context
    let context = get_app_context();
    context.js_engine.hap_map.insert(aid, recv);
    // 执行js
    match context.js_engine
        .execute_side_module(ExecuteSideModuleParam::new(1,url)).await
        .tap_err(|e| error!("执行js错误")) {
        Ok(_) => {
            // 注册到manage 上
            // self.mapping_js_map.insert(cid, module);
            // Ok(ch_sender.clone())
            Ok(sender)
        }
        Err(_) => {
            //删除掉
            context.js_engine.hap_map.remove(&aid);
            Err(anyhow::anyhow!("执行js错误"))
        }
    }
}


fn init_js_files(aid: i64, script: &str) -> anyhow::Result<Url> {
    let md5 = md5::compute(script.as_bytes());

    let md5_hdx = hex::encode(md5.0);
    let context = get_app_context();
    let dir = context.config.server.data_dir.as_str();
    let js_file_str = format!("{}/js_scripts/hap/{}_{}/handle.js", dir, aid, md5_hdx);
    let js_file = PathBuf::from(js_file_str.as_str());
    fs::create_dir_all(js_file.parent().unwrap())?;
    fs::write(js_file.as_path(), script)?;
    let url = Url::parse(js_file_str.as_str())?;
    Ok(url)
}