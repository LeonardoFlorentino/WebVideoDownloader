#[derive(Clone, serde::Serialize)]
pub struct ProgressPayload {
    pub id: String,
    pub progress: u64,
    pub total: u64,
    pub status: String,
}
use tauri::{AppHandle, State};
use std::sync::Arc;
use crate::backend::download_manager::DownloadManager;
use serde::Serialize;

#[tauri::command]
pub async fn resume_download(
    app: AppHandle,
    state: State<'_, Arc<DownloadManager>>,
    id: String,
    username: String,
    url: String,
    save_path: String,
    progress_key: Option<String>,
    source: Option<String>,
) -> Result<(), String> {
    crate::commands::video::start_download(
        app,
        state,
        id,
        username,
        url,
        save_path,
        progress_key,
        source,
    )
    .await
}

#[tauri::command]
pub fn pause_download(
    state: State<'_, Arc<DownloadManager>>,
    id: String,
) {
    let mut downloads = state.downloads.lock().unwrap();
    if let Some(task) = downloads.get_mut(&id) {
        task.handle.abort();
        // Opcional: emitir evento de pausa
    }
}

#[derive(Serialize)]
pub struct CommandResult<T> {
    pub ok: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}