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
    #[serde(default)]
    pub scope: Option<String>,
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

pub fn update_progress(key: &str, progress: DownloadProgress) {
    let mut all = load_all_progress();
    all.insert(key.to_string(), progress.clone());
    save_all_progress(&all);
    // Log detalhado
    // log removido

    // Persistir porcentagem no user.json sempre que houver progresso
    let should_sync_main_url = !matches!(progress.scope.as_deref(), Some("panel"));
    if should_sync_main_url {
        if let Some(username) = crate::backend::user_service::get_username_for_url(&progress.url) {
        let total = progress.total_size as f32;
        let downloaded = progress.downloaded as f32;
        let percent = if total > 0.0 { downloaded / total } else { 0.0 };
            let _ = crate::backend::user_service::update_main_url_progress(
                username,
                progress.url.clone(),
                percent,
            );
        }
    }
}

pub fn get_progress(key: &str) -> Option<DownloadProgress> {
    use std::collections::HashSet;
    use std::sync::Mutex;
    use once_cell::sync::Lazy;
    static LOGGED_MISSING: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));
    let all = load_all_progress();
    let found = all.get(key);
    if let Some(p) = found {
        // log removido
        return Some(p.clone());
    }
    if !key.starts_with("http://") && !key.starts_with("https://") {
        return None;
    }
    // Se não há progresso salvo, mas o arquivo existe e está completo, retorna progresso concluído
    // Tenta deduzir nome do arquivo salvo a partir da URL real
    let root = crate::backend::filesystem::get_project_root();
    let videos_dir = root.join("Vídeos baixados");
    // Extrai o nome real do arquivo da URL
    let filename_from_url = key.split('/').last().unwrap_or("");
    let file_path = videos_dir.join(filename_from_url);
    if file_path.exists() {
        let meta = std::fs::metadata(&file_path).ok();
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        if size > 0 {
            let progress = DownloadProgress {
                url: key.to_string(),
                filename: filename_from_url.to_string(),
                total_size: size,
                downloaded: size,
                status: "concluído".to_string(),
                id: None,
                scope: Some("home".to_string()),
            };
            // log removido
            return Some(progress);
        }
    }
    // Só loga uma vez por URL
    let mut logged = LOGGED_MISSING.lock().unwrap();
    if !logged.contains(key) {
        // log removido
        logged.insert(key.to_string());
    }
    None
}

pub fn remove_progress(key: &str) {
    let mut all = load_all_progress();
    all.remove(key);
    save_all_progress(&all);
}
