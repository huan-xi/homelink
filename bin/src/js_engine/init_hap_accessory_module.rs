use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use anyhow::anyhow;
use deno_runtime::deno_core::url::Url;
use log::error;
use tap::TapFallible;
use crate::config::context::get_app_context;
use crate::db::SNOWFLAKE;
use crate::init::hap_manage::{HapManage};
use crate::js_engine::channel::params::ExecuteSideModuleParam;

/// hap 设备作为一个module 运行

pub async fn init_hap_accessory_module(manage: HapManage, aid: u64, script: &str) -> anyhow::Result<()> {
    // let (sender, recv, exit_ch) = hap_channel::channel();
    //注册到context
    let url = init_js_files(aid, script)?;
    //注册到context
    let context = get_app_context();
    // context.js_engine.hap_map.insert(aid, recv);
    let id = SNOWFLAKE.next_id();
    // 执行js
    match context.js_engine
        .execute_side_module(ExecuteSideModuleParam::new(id, url)).await
        .tap_err(|e| error!("执行js错误"))
    {
        Ok(_) => {
            // 注册到manage 上
            manage.put_accessory_ch(aid, id).await;
            Ok(())
        }
        Err(e) => {
            //删除掉
            // context.js_engine.hap_map.remove(&aid);
            Err(anyhow::anyhow!("执行js模块错误,{}", e))
        }
    }
}


const FILE_PROTOCOL: &str = "file://";

fn init_js_files(aid: u64, script: &str) -> anyhow::Result<Url> {
    if script.starts_with(FILE_PROTOCOL) {
        let path = script.replace(FILE_PROTOCOL, "");
        let url = Url::from_file_path(path.as_str())
            .map_err(|r| anyhow!("设置的路径错误"))?;
        return Ok(url);
    }
    let md5 = md5::compute(script.as_bytes());
    let md5_hdx = hex::encode(md5.0);
    let context = get_app_context();
    let dir = context.config.server.data_dir.as_str();
    let js_file_str = format!("{}/js_scripts/hap/{}_{}/handle.js", dir, aid, md5_hdx);
    let js_file = PathBuf::from(js_file_str.as_str());
    fs::create_dir_all(js_file.parent().unwrap())?;
    fs::write(js_file.as_path(), script)?;
    let url = Url::from_file_path(js_file_str.as_str())
        .map_err(|r| anyhow!("url 解析错误"))?;
    Ok(url)
}