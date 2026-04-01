#[allow(dead_code)]
// Handles HLS and JMV downloads with window
#[tauri::command]
pub fn download_special_video(username: String, url: String, save_path: String, id: Option<u64>) -> CommandResult<()> {
    let filename = std::path::Path::new(&save_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("video.mp4").to_string();
    if url.ends_with(".m3u8") || url.contains(".m3u8?") || url.contains("player.jmvstream.com") {
        let username_clone = username.clone();
        let url_clone = url.clone();
        let filename_clone = filename.clone();
        let id_clone = id.clone();
        std::thread::spawn(move || {
            let client = crate::backend::http_client_helper::create_blocking_client().unwrap();
            let mut path = crate::backend::filesystem::get_project_root();
            path.push("Vídeos baixados");
            std::fs::create_dir_all(&path).ok();
            path.push(&filename_clone);
            let _ = crate::backend::download_helpers::baixar_jmvstream(&client, &url_clone, &path, None, id_clone, Some(&username_clone), None);
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
pub fn get_progress_command(url: String) -> GetProgressResult {
    match crate::backend::download_progress::get_progress(&url) {
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
    app: tauri::AppHandle,
) -> Result<(), String> {
    println!(
        "[PAUSE_INTEGRATED] requisicao recebida id='{}' url='{}'",
        id, url
    );

    // Atualiza progresso para "pausado"
    if let Some(mut prog) = crate::backend::download_progress::get_progress(&url) {
        prog.status = "pausado".to_string();
        crate::backend::download_progress::update_progress(&url, prog);
    }
    // Seta flag global de pausa para downloads HLS
    {
        use crate::backend::download_helpers::PAUSE_FLAGS;
        let mut map = PAUSE_FLAGS.lock().unwrap();
        let flag = map
            .entry(url.clone())
            .or_insert_with(|| std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)))
            .clone();
        flag.store(true, std::sync::atomic::Ordering::SeqCst);
        println!(
            "[PAUSE_INTEGRATED] flag setada como TRUE para url='{}'",
            url
        );
    }
    // Aborta a task protegida pelo lock, mas solta o lock antes de emitir evento
    let mut found_task = false;
    {
        let mut downloads = state.downloads.lock().unwrap();
        if let Some(task) = downloads.get_mut(&id) {
            found_task = true;
            task.handle.abort();
        }
    }
    println!(
        "[PAUSE_INTEGRATED] id='{}' url='{}' task_encontrada={}",
        id, url, found_task
    );
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
) -> Result<(), String> {
    // Limpa eventual flag antiga de pausa para esta URL antes de iniciar/retomar.
    {
        use crate::backend::download_helpers::PAUSE_FLAGS;
        let map = PAUSE_FLAGS.lock().unwrap();
        if let Some(flag) = map.get(&url) {
            flag.store(false, std::sync::atomic::Ordering::SeqCst);
        }
    }

    // Always ensure MainUrl entry exists before download
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

    // If HLS or JMV, return error to frontend to call download_special_video
    if url.ends_with(".m3u8") || url.contains(".m3u8?") || url.contains("player.jmvstream.com") {
        return Err("special_video".to_string());
    }

    // Fluxo padrão para arquivos diretos
    // Garante que sempre exista progresso inicial salvo
    if crate::backend::download_progress::get_progress(&url).is_none() {
        let progress = crate::backend::download_progress::DownloadProgress {
            url: url.clone(),
            filename: std::path::Path::new(&save_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("video.mp4").to_string(),
            total_size: 0,
            downloaded: 0,
            status: "baixando".to_string(),
            id: None,
        };
        crate::backend::download_progress::update_progress(&url, progress);
    }
    let manager = state.inner().clone();
    let id_clone = id.clone();
    let username_clone = username.clone();
    let url_clone = url.clone();
    let save_path_clone = save_path.clone();
    let app_clone = app.clone();
    let handle = tokio::spawn(async move {
        // Flag de pausa por URL (tambem para download direto).
        let pause_flag = {
            use crate::backend::download_helpers::PAUSE_FLAGS;
            let mut map = PAUSE_FLAGS.lock().unwrap();
            map.entry(url_clone.clone())
                .or_insert_with(|| {
                    std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false))
                })
                .clone()
        };

            println!("[START_DOWNLOAD] INICIANDO download para url='{}' filename='{}' username='{}'", url_clone, save_path_clone, username_clone);
        use futures_util::StreamExt;
        use std::io::Write;
        // Se o arquivo já existe mas é de uma URL diferente (sem progresso salvo = novo download),
        // gera um nome único para não sobrescrever o arquivo concluído.
        let save_path_clone: String = {
            let p = std::path::Path::new(&save_path_clone);
            let has_progress = crate::backend::download_progress::get_progress(&url_clone).is_some();
            if p.exists() && !has_progress {
                let unique = crate::backend::download_helpers::unique_save_path(p);
                println!(
                    "[START_DOWNLOAD] arquivo '{}' já existe, usando nome único '{}'",
                    p.display(), unique.display()
                );
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
        println!("[START_DOWNLOAD][HTTP] Status: {}", response.status());
        let content_length = response.content_length().unwrap_or(0);
        println!("[START_DOWNLOAD][HTTP] Content-Length: {}", content_length);
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
        let mut last_terminal_log = std::time::Instant::now();
        while let Some(chunk) = stream.next().await {
            // Interrompe definitivamente quando pausado (nao cai em completed).
            if pause_flag.load(std::sync::atomic::Ordering::SeqCst) {
                let _ = crate::backend::user_service::update_main_url_status(
                    username_clone.clone(),
                    url_clone.clone(),
                    "pausado".to_string(),
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
                    println!("[START_DOWNLOAD][ERRO] erro ao receber chunk: {}", e);
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
                    println!("[START_DOWNLOAD][ERRO] erro ao escrever chunk: {}", e);
                    let _ = app_clone.emit_to("all", "download_finished", serde_json::json!({
                        "url": url_clone,
                        "status": "erro",
                        "error": e.to_string()
                    }));
                    return;
                }
                write_buffer.clear();
            }
            if last_terminal_log.elapsed().as_secs() >= 5 {
                let percent = if total_size > 0 {
                    ((downloaded as f64 / total_size as f64) * 100.0).clamp(0.0, 100.0)
                } else {
                    0.0
                };
                println!(
                    "[START_DOWNLOAD][PROGRESSO] id='{}' {:.1}% ({}/{}) url='{}'",
                    id_clone,
                    percent,
                    downloaded,
                    total_size,
                    url_clone
                );
                last_terminal_log = std::time::Instant::now();
            }
            if last_emit.elapsed().as_millis() > 500 {
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
                let _ = app_clone.emit_to("all", "download-progress", super::download_manager::ProgressPayload {
                    id: id_clone.clone(),
                    progress: downloaded,
                    total: total_size,
                    status: "downloading".to_string(),
                });
                last_emit = std::time::Instant::now();
            }
        }
        // Checagem final de pausa antes de concluir.
        if pause_flag.load(std::sync::atomic::Ordering::SeqCst) {
            let _ = crate::backend::user_service::update_main_url_status(
                username_clone.clone(),
                url_clone.clone(),
                "pausado".to_string(),
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
        println!("[START_DOWNLOAD] FINALIZADO download para url='{}' filename='{}' username='{}' tamanho={} bytes", url_clone, save_path_clone, username_clone, downloaded);
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
    println!("[TAURI][download_video] chamado para url='{}' filename='{}' username='{}'", url, filename, username);
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
                crate::backend::download_helpers::baixar_jmvstream(&client, &url_clone, &path, Some(&window_clone), id, Some(&username_clone), None)
            } else if url_clone.ends_with(".m3u8") || url_clone.contains(".m3u8?") {
                let client = crate::backend::http_client_helper::create_blocking_client().unwrap();
                let mut path = crate::backend::filesystem::get_project_root();
                path.push("Vídeos baixados");
                std::fs::create_dir_all(&path).ok();
                path.push(&filename_clone);
                crate::backend::download_helpers::baixar_jmvstream(&client, &url_clone, &path, Some(&window_clone), id, Some(&username_clone), None)
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
