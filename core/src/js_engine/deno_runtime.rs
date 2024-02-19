use std::path::PathBuf;
use std::rc::Rc;
use anyhow::anyhow;
use deno_runtime::deno_core;
use deno_runtime::deno_core::{FsModuleLoader, ModuleSpecifier};
use deno_runtime::permissions::PermissionsContainer;
use deno_runtime::worker::{MainWorker, WorkerOptions};
use log::info;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use crate::js_engine::context::EnvContext;
use crate::js_engine::ext::env;
use crate::js_engine::scripts::ScriptAsset;


pub async fn start_deno_runtime(context: EnvContext, dir: String, js_path: PathBuf) ->anyhow::Result<()>{
    let path_buf = if let Err(e) = fs::metadata(js_path.as_path()).await {
        let embed_file = ScriptAsset::get("main.js")
            .unwrap();
        fs::create_dir_all(dir).await?;
        info!("js path:{:?}", js_path.clone().canonicalize());
        let mut file = fs::File::create(&js_path).await?;
        file.write_all(embed_file.data.as_ref()).await?;
        js_path.canonicalize()?
    } else {
        js_path.clone().canonicalize()?
    };
    info!("js path:{:?}", path_buf);

    let main_module = ModuleSpecifier::from_file_path(path_buf.as_path())
        .map_err(|e| anyhow!("main.js 不存在"))?;
    let ops_env = env::deno_env::init_ops_and_esm(context);
    let mut worker = MainWorker::bootstrap_from_options(
        main_module.clone(),
        PermissionsContainer::allow_all(),
        WorkerOptions {
            module_loader: Rc::new(FsModuleLoader) as Rc<dyn deno_core::ModuleLoader>,
            extensions: vec![ops_env],
            ..Default::default()
        },
    );

    worker.execute_main_module(&main_module).await?;
    let even_loop = worker.run_event_loop(false);
    even_loop.await?;
    Ok(())
}