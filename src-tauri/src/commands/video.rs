#[allow(dead_code)]
// Handles HLS and JMV downloads with window
#[tauri::command]
pub fn download_special_video(
    username: String,
    url: String,
    save_path: String,
    id: Option<u64>,
    progress_key: Option<String>,
    source: Option<String>,
) -> CommandResult<()> {
    if url.ends_with(".m3u8") || url.contains(".m3u8?") || url.contains("player.jmvstream.com") {
        let username_clone = username.clone();
        let url_clone = url.clone();
        let save_path_clone = save_path.clone();
        let id_clone = id;
        let progress_key_clone = progress_key.unwrap_or_else(|| url.clone());
        let source_clone = source.unwrap_or_else(|| "home".to_string());
        std::thread::spawn(move || {
            let client = crate::backend::http_client_helper::create_blocking_client().unwrap();
            let path = std::path::PathBuf::from(&save_path_clone);
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = crate::backend::download_helpers::baixar_jmvstream(
                &client,
                &url_clone,
                &path,
                None,
                id_clone,
                Some(&username_clone),
                None,
                Some(&progress_key_clone),
                Some(&source_clone),
            );
        });
        return CommandResult { ok: true, data: Some(()), error: None };
    }
    CommandResult { ok: false, data: None, error: Some("Invalid special video type".to_string()) }
}
use serde::Serialize;
#[derive(Serialize)]
pub struct GetProgressResult {
    pub ok: bool,
    pub data: Option<crate::backend::download_progress::DownloadProgress>,
    pub error: Option<String>,
}

#[tauri::command]
pub fn get_progress_command(url: Option<String>, progress_key: Option<String>) -> GetProgressResult {
    let lookup_key = progress_key.or(url.clone());

    match lookup_key
        .as_deref()
        .and_then(crate::backend::download_progress::get_progress)
    {
        Some(progress) => GetProgressResult {
            ok: true,
            data: Some(progress),
            error: None,
        },
        None => GetProgressResult {
            ok: false,
            data: None,
            error: Some("No progress found for URL".to_string()),
        },
    }
}
#[cfg(test)]
mod tests {
    // Testes removidos temporariamente devido a dependências não resolvidas
}
use crate::backend::download_manager::{DownloadManager, DownloadTask};
use crate::commands::download_manager::CommandResult;
use tauri::Window;
use tauri::Emitter;
use crate::backend::downloads::baixar_video_emit;
// Integrated pause download (moved from main.rs)
#[tauri::command(rename = "integrated_pause_download", async)]
pub async fn integrated_pause_download(
    state: tauri::State<'_, std::sync::Arc<DownloadManager>>,
    id: String,
    url: String,
    progress_key: Option<String>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let progress_key = progress_key.unwrap_or_else(|| url.clone());

    // Atualiza progresso para "pausado"
    if let Some(mut prog) = crate::backend::download_progress::get_progress(&progress_key) {
        prog.status = "pausado".to_string();
        crate::backend::download_progress::update_progress(&progress_key, prog);
    }
    // Seta flag global de pausa para downloads HLS
    {
        use crate::backend::download_helpers::PAUSE_FLAGS;
        let mut map = PAUSE_FLAGS.lock().unwrap();
        let flag = map
            .entry(progress_key.clone())
            .or_insert_with(|| std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)))
            .clone();
        flag.store(true, std::sync::atomic::Ordering::SeqCst);
    }
    // Aborta a task protegida pelo lock, mas solta o lock antes de emitir evento
    {
        let mut downloads = state.downloads.lock().unwrap();
        if let Some(task) = downloads.get_mut(&id) {
            task.handle.abort();
        }
    }
    // Só emite evento depois de liberar o lock
    let _ = app.emit_to("all", "download_paused", serde_json::json!({ "url": url }));
    Ok(())
}

// Start download (moved from main.rs)
#[tauri::command]
pub async fn start_download(
    app: tauri::AppHandle,
    state: tauri::State<'_, std::sync::Arc<DownloadManager>>,
    id: String,
    username: String,
    url: String,
    save_path: String,
    progress_key: Option<String>,
    source: Option<String>,
) -> Result<(), String> {
    let progress_key = progress_key.unwrap_or_else(|| url.clone());
    let source = source.unwrap_or_else(|| "home".to_string());

    // Limpa eventual flag antiga de pausa para esta URL antes de iniciar/retomar.
    {
        use crate::backend::download_helpers::PAUSE_FLAGS;
        let map = PAUSE_FLAGS.lock().unwrap();
        if let Some(flag) = map.get(&progress_key) {
            flag.store(false, std::sync::atomic::Ordering::SeqCst);
        }
    }

    if source == "home" {
        let _ = crate::backend::user_service::add_main_url(
            username.clone(),
            url.clone(),
            Some(
                std::path::Path::new(&save_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("video.mp4")
                    .to_string(),
            ),
        );
    }

    // If HLS or JMV, return error to frontend to call download_special_video
    if url.ends_with(".m3u8") || url.contains(".m3u8?") || url.contains("player.jmvstream.com") {
        return Err("special_video".to_string());
    }

    // Fluxo padrão para arquivos diretos
    if crate::backend::download_progress::get_progress(&progress_key).is_none() {
        let progress = crate::backend::download_progress::DownloadProgress {
            url: url.clone(),
            filename: std::path::Path::new(&save_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("video.mp4").to_string(),
            total_size: 0,
            downloaded: 0,
            status: "baixando".to_string(),
            id: id.parse::<u64>().ok(),
            scope: Some(source.clone()),
        };
        crate::backend::download_progress::update_progress(&progress_key, progress);
    }
    let manager = state.inner().clone();
    let id_clone = id.clone();
    let username_clone = username.clone();
    let url_clone = url.clone();
    let save_path_clone = save_path.clone();
    let progress_key_clone = progress_key.clone();
    let source_clone = source.clone();
    let app_clone = app.clone();
    let handle = tokio::spawn(async move {
        let filename = std::path::Path::new(&save_path_clone)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("video.mp4")
            .to_string();
        let progress_id = id_clone.parse::<u64>().ok();
        let build_progress = |downloaded: u64, total_size: u64, status: &str| {
            crate::backend::download_progress::DownloadProgress {
                url: url_clone.clone(),
                filename: filename.clone(),
                total_size,
                downloaded,
                status: status.to_string(),
                id: progress_id,
                scope: Some(source_clone.clone()),
            }
        };

        // Flag de pausa por chave lógica do download.
        let pause_flag = {
            use crate::backend::download_helpers::PAUSE_FLAGS;
            let mut map = PAUSE_FLAGS.lock().unwrap();
            map.entry(progress_key_clone.clone())
                .or_insert_with(|| {
                    std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false))
                })
                .clone()
        };
        use futures_util::StreamExt;
        use std::io::Write;
        let save_path_clone: String = {
            let p = std::path::Path::new(&save_path_clone);
            let has_progress = crate::backend::download_progress::get_progress(&progress_key_clone).is_some();
            if p.exists() && !has_progress {
                let unique = crate::backend::download_helpers::unique_save_path(p);
                unique.to_string_lossy().into_owned()
            } else {
                save_path_clone.clone()
            }
        };
        let current_size = if std::path::Path::new(&save_path_clone).exists() {
            std::fs::metadata(&save_path_clone).map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };
        let client = match crate::backend::http_client_helper::create_async_client() {
            Ok(c) => c,
            Err(e) => {
                let _ = app_clone.emit_to("all", "download_finished", serde_json::json!({
                    "url": url_clone,
                    "status": "erro",
                    "error": e.to_string()
                }));
                return;
            }
        };
        let mut request = client.get(&url_clone);
        if current_size > 0 {
            request = request.header("Range", format!("bytes={}-", current_size));
        }
        let response = match request.send().await {
            Ok(r) => r,
            Err(e) => {
                let _ = app_clone.emit_to("all", "download_finished", serde_json::json!({
                    "url": url_clone,
                    "status": "erro",
                    "error": e.to_string()
                }));
                return;
            }
        };
                let mut total_size = response.content_length()
            .or_else(|| {
                response.headers().get("content-range")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.split('/').nth(1))
                    .and_then(|n| n.parse().ok())
            })
            .unwrap_or(0);
        if total_size == 0 {
            if let Ok(head_resp) = reqwest::Client::new().head(&url_clone).send().await {
                total_size = head_resp.content_length().unwrap_or(0);
            }
        }
        if let Some(parent) = std::path::Path::new(&save_path_clone).parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                let _ = app_clone.emit_to("all", "download_finished", serde_json::json!({
                    "url": url_clone,
                    "status": "erro",
                    "error": e.to_string()
                }));
                return;
            }
        }
        let mut file = match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&save_path_clone)
        {
            Ok(f) => f,
            Err(e) => {
                let _ = app_clone.emit_to("all", "download_finished", serde_json::json!({
                    "url": url_clone,
                    "status": "erro",
                    "error": e.to_string()
                }));
                return;
            }
        };
        let mut stream = response.bytes_stream();
        let mut downloaded = current_size;
        let mut write_buffer = Vec::with_capacity(64 * 1024);
        let mut last_emit = std::time::Instant::now();
                while let Some(chunk) = stream.next().await {
            if pause_flag.load(std::sync::atomic::Ordering::SeqCst) {
                if source_clone == "home" {
                    let _ = crate::backend::user_service::update_main_url_status(
                        username_clone.clone(),
                        url_clone.clone(),
                        "pausado".to_string(),
                    );
                }
                crate::backend::download_progress::update_progress(
                    &progress_key_clone,
                    build_progress(downloaded, total_size, "pausado"),
                );
                let _ = app_clone.emit_to("all", "download-progress", super::download_manager::ProgressPayload {
                    id: id_clone.clone(),
                    progress: downloaded,
                    total: total_size,
                    status: "paused".to_string(),
                });
                let _ = app_clone.emit_to("all", "download_paused", serde_json::json!({ "url": url_clone }));
                return;
            }

            let chunk = match chunk {
                Ok(c) => c,
                Err(e) => {
                    crate::backend::download_progress::update_progress(
                        &progress_key_clone,
                        build_progress(downloaded, total_size, "erro"),
                    );
                    let _ = app_clone.emit_to("all", "download_finished", serde_json::json!({
                        "url": url_clone,
                        "status": "erro",
                        "error": e.to_string()
                    }));
                    return;
                }
            };
            write_buffer.extend_from_slice(&chunk);
            downloaded += chunk.len() as u64;
            if write_buffer.len() >= 64 * 1024 {
                if let Err(e) = file.write_all(&write_buffer) {
                    crate::backend::download_progress::update_progress(
                        &progress_key_clone,
                        build_progress(downloaded, total_size, "erro"),
                    );
                    let _ = app_clone.emit_to("all", "download_finished", serde_json::json!({
                        "url": url_clone,
                        "status": "erro",
                        "error": e.to_string()
                    }));
                    return;
                }
                write_buffer.clear();
            }
            
            if last_emit.elapsed().as_millis() > 500 {
                crate::backend::download_progress::update_progress(
                    &progress_key_clone,
                    build_progress(downloaded, total_size, "baixando"),
                );
                if source_clone == "home" {
                    let progress = if total_size > 0 {
                        downloaded as f32 / total_size as f32
                    } else {
                        0.0
                    };
                    let _ = crate::backend::user_service::update_main_url_progress(
                        username_clone.clone(),
                        url_clone.clone(),
                        progress,
                    );
                }
                let _ = app_clone.emit_to("all", "download-progress", super::download_manager::ProgressPayload {
                    id: id_clone.clone(),
                    progress: downloaded,
                    total: total_size,
                    status: "downloading".to_string(),
                });
                last_emit = std::time::Instant::now();
            }
        }
        if pause_flag.load(std::sync::atomic::Ordering::SeqCst) {
            if source_clone == "home" {
                let _ = crate::backend::user_service::update_main_url_status(
                    username_clone.clone(),
                    url_clone.clone(),
                    "pausado".to_string(),
                );
            }
            crate::backend::download_progress::update_progress(
                &progress_key_clone,
                build_progress(downloaded, total_size, "pausado"),
            );
            let _ = app_clone.emit_to("all", "download-progress", super::download_manager::ProgressPayload {
                id: id_clone.clone(),
                progress: downloaded,
                total: total_size,
                status: "paused".to_string(),
            });
            let _ = app_clone.emit_to("all", "download_paused", serde_json::json!({ "url": url_clone }));
            return;
        }

        if !write_buffer.is_empty() {
            let _ = file.write_all(&write_buffer);
        }
        crate::backend::download_progress::update_progress(
            &progress_key_clone,
            build_progress(downloaded, total_size, "concluído"),
        );
        if source_clone == "home" {
            let progress = if total_size > 0 {
                downloaded as f32 / total_size as f32
            } else {
                1.0
            };
            let _ = crate::backend::user_service::update_main_url_progress(
                username_clone.clone(),
                url_clone.clone(),
                progress,
            );
        }
        let _ = app_clone.emit_to("all", "download-progress", super::download_manager::ProgressPayload {
            id: id_clone,
            progress: downloaded,
            total: total_size,
            status: "completed".to_string(),
        });
    });
    let mut downloads = manager.downloads.lock().unwrap();
    downloads.insert(id.clone(), DownloadTask {
        handle,
    });
    Ok(())
}

#[tauri::command]
pub fn download_video(window: Window, username: String, url: String, filename: String, id: Option<u64>) -> CommandResult<()> {
    let window_clone = window.clone();
    let username_clone = username.clone();
    let url_clone = url.clone();
    let filename_clone = filename.clone();
    std::thread::spawn(move || {
        let result = if url_clone.contains("player.jmvstream.com") {
                let client = crate::backend::http_client_helper::create_blocking_client().unwrap();
                let mut path = crate::backend::filesystem::get_project_root();
                path.push("Vídeos baixados");
                std::fs::create_dir_all(&path).ok();
                path.push(&filename_clone);
                crate::backend::download_helpers::baixar_jmvstream(&client, &url_clone, &path, Some(&window_clone), id, Some(&username_clone), None, None, None)
            } else if url_clone.ends_with(".m3u8") || url_clone.contains(".m3u8?") {
                let client = crate::backend::http_client_helper::create_blocking_client().unwrap();
                let mut path = crate::backend::filesystem::get_project_root();
                path.push("Vídeos baixados");
                std::fs::create_dir_all(&path).ok();
                path.push(&filename_clone);
                crate::backend::download_helpers::baixar_jmvstream(&client, &url_clone, &path, Some(&window_clone), id, Some(&username_clone), None, None, None)
            } else {
                baixar_video_emit(Some(&window_clone), &url_clone, &filename_clone)
            };
        let _ = match result {
            Ok(_) => {
                let _ = crate::backend::user_service::update_main_url_status(
                    username_clone.clone(),
                    url_clone.clone(),
                    "concluído".to_string(),
                );
                let _ = window_clone.emit("download_finished", serde_json::json!({ "url": url_clone, "filename": filename_clone, "status": "concluido" }));
            },
            Err(e) => {
                let _ = window_clone.emit("download_finished", serde_json::json!({ "url": url_clone, "filename": filename_clone, "status": "erro", "error": e }));
            },
        };
    });
    CommandResult { ok: true, data: Some(()), error: None }
}
