use std::collections::HashMap;
use std::sync::Mutex;

pub struct DownloadTask {
    pub handle: tokio::task::JoinHandle<()>,
}

pub struct DownloadManager {
    pub downloads: Mutex<HashMap<String, DownloadTask>>,
}

impl Default for DownloadManager {
    fn default() -> Self {
        DownloadManager {
            downloads: Mutex::new(HashMap::new()),
        }
    }
}
