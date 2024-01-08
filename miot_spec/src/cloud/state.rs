use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;
use log::{error, warn};
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};

#[derive(Debug)]
pub struct State {
    cookie_store_path: PathBuf,
    pub cookie_store: Arc<CookieStoreMutex>,
}

impl State {
    pub fn try_new(cookie_store_path: PathBuf) -> anyhow::Result<State> {
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

        Ok(State {
            cookie_store_path,
            cookie_store,
        })
    }
}

impl Drop for State {
    fn drop(&mut self) {
        let mut file = match fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.cookie_store_path)
        {
            Ok(f) => f,
            Err(e) => {
                error!(
                    "open {} for write failed. error: {}",
                    self.cookie_store_path.display(),
                    e
                );
                return;
            }
        };

        let store = self.cookie_store.lock().unwrap();
        if let Err(e) = store.save_json(&mut file) {
            error!(
                "save cookies to path {} failed. error: {}",
                self.cookie_store_path.display(),
                e
            );
        }
    }
}