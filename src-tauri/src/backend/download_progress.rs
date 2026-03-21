use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DownloadProgress {
    pub url: String,
    pub filename: String,
    pub total_size: u64,
    pub downloaded: u64,
    pub status: String, // "baixando", "pausado", "concluido", "erro"
}

pub fn get_progress_file() -> PathBuf {
    let mut path = crate::backend::filesystem::get_project_root();
    path.push("user_data/download_progress.json");
    path
}

pub fn load_all_progress() -> HashMap<String, DownloadProgress> {
    let path = get_progress_file();
    if let Ok(data) = fs::read_to_string(&path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        HashMap::new()
    }
}

pub fn save_all_progress(map: &HashMap<String, DownloadProgress>) {
    use std::io::Write;
    let path = get_progress_file();
    if let Ok(json) = serde_json::to_string_pretty(map) {
        let _ = fs::create_dir_all(path.parent().unwrap());
        if let Ok(mut file) = std::fs::File::create(&path) {
            let _ = file.write_all(json.as_bytes());
            let _ = file.flush();
        }
    }
}

pub fn update_progress(url: &str, progress: DownloadProgress) {
    let mut all = load_all_progress();
    use std::time::{SystemTime, UNIX_EPOCH};
    static mut LAST_UPDATE_LOG: u128 = 0;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    if progress.status == "baixando" {
        let mut should_print = false;
        unsafe {
            if now - LAST_UPDATE_LOG > 5000 {
                should_print = true;
                LAST_UPDATE_LOG = now;
            }
        }
        if should_print {
            println!("[UPDATE_PROGRESS] Salvando status 'baixando' para URL: {}", url);
        }
    } else {
        println!("[UPDATE_PROGRESS] Salvando status '{}' para URL: {}", progress.status, url);
    }
    all.insert(url.to_string(), progress);
    save_all_progress(&all);
}

pub fn get_progress(url: &str) -> Option<DownloadProgress> {
    use std::time::{SystemTime, UNIX_EPOCH};
    static mut LAST_PRINT: u128 = 0;
    let all = load_all_progress();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let mut should_print = false;
    unsafe {
        if now - LAST_PRINT > 5000 {
            should_print = true;
            LAST_PRINT = now;
        }
    }
    if should_print {
        println!("[GET_PROGRESS] Buscando progresso para URL: {}. URLs existentes: {:?}", url, all.keys().collect::<Vec<_>>() );
    }
    all.get(url).cloned()
}

pub fn remove_progress(url: &str) {
    let mut all = load_all_progress();
    all.remove(url);
    save_all_progress(&all);
}
