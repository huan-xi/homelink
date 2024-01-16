use std::fs;
use std::fs::{create_dir_all, File};
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use log::{error, warn};
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};

#[derive(Debug)]
pub struct CookieState {
    cookie_store_path: PathBuf,

    pub cookie_store: Arc<CookieStoreMutex>,
}

impl CookieState {
    pub fn try_new(cookie_store_path: PathBuf) -> anyhow::Result<CookieState> {




        let cookie_store = match File::open(&cookie_store_path) {
            Ok(f) => CookieStore::load_json(BufReader::new(f)).map_err(|e| {
                let context = format!(
                    "error when read cookies from {}",
                    cookie_store_path.display()
                );
                anyhow::anyhow!("{}", e).context(context)
            })?,
            Err(e) => {
                warn!(
                    "open {} failed. error: {}, use default empty cookie store",
                    cookie_store_path.display(),
                    e
                );
                CookieStore::default()
            }
        };
        let cookie_store = Arc::new(CookieStoreMutex::new(cookie_store));

        Ok(CookieState {
            cookie_store_path,
            cookie_store,
        })
    }
}

impl CookieState {
    pub fn get_cookie_store(&self) -> Arc<CookieStoreMutex> {
        Arc::clone(&self.cookie_store)
    }
    pub fn save(&self) -> anyhow::Result<()> {
        //如果文件不存在则创建


        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.cookie_store_path);
        let mut file = if let Err(e) = file {
            create_dir_all(&self.cookie_store_path.parent()
                .ok_or(anyhow::anyhow!("路径错误"))?)?;
            match File::create(&self.cookie_store_path) {
                Ok(f) => f,
                Err(e) => {
                    error!("create {} for write failed. error: {}",
                            self.cookie_store_path.display(),
                            e);
                    return Err(anyhow::anyhow!(
                        "create {} for write failed. error: {}",
                        self.cookie_store_path.display(),
                        e
                    ));
                }
            }
        } else { file.unwrap() };


        let store = self.cookie_store.lock().unwrap();
        store.save_json(&mut file).map_err(|e| {
            let context = format!(
                "error when save cookies to {}",
                self.cookie_store_path.display()
            );
            anyhow::anyhow!("{}", e).context(context)
        })
    }
}

impl Drop for CookieState {
    fn drop(&mut self) {
        let _ = self.save();
    }
}