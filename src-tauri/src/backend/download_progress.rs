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
    pub status: String, // "baixando", "pausado", "concluído", "erro"
    pub id: Option<u64>, // id numérico para eventos
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
    all.insert(url.to_string(), progress.clone());
    save_all_progress(&all);
    // Log detalhado
    println!("[BACKEND][update_progress] url='{}' status='{}' downloaded={} total={} filename='{}' id={:?}", url, progress.status, progress.downloaded, progress.total_size, progress.filename, progress.id);
}

pub fn get_progress(url: &str) -> Option<DownloadProgress> {
    let all = load_all_progress();
    let found = all.get(url);
    if let Some(p) = found {
        println!("[BACKEND][get_progress] url='{}' status='{}' downloaded={} total={} filename='{}' id={:?}", url, p.status, p.downloaded, p.total_size, p.filename, p.id);
    } else {
        println!("[BACKEND][get_progress] url='{}' NÃO ENCONTRADO", url);
    }
    found.cloned()
}

pub fn remove_progress(url: &str) {
    let mut all = load_all_progress();
    all.remove(url);
    save_all_progress(&all);
}
