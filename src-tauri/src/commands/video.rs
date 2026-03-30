#[allow(dead_code)]
// Handles HLS and JMV downloads with window
#[tauri::command]
pub fn download_special_video(window: tauri::Window, username: String, url: String, save_path: String, id: Option<u64>) -> CommandResult<()> {
    let filename = std::path::Path::new(&save_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("video.mp4").to_string();
    if url.ends_with(".m3u8") || url.contains(".m3u8?") {
        let result = baixar_hls_emit(&window, &username, &url, &filename, id);
        return match result {
            Ok(_) => CommandResult { ok: true, data: Some(()), error: None },
            Err(e) => CommandResult { ok: false, data: None, error: Some(e) },
        };
    } else if url.contains("player.jmvstream.com") {
        let result = baixar_player_jmvstream(&window, &username, &url, &filename, id);
        return match result {
            Ok(_) => CommandResult { ok: true, data: Some(()), error: None },
            Err(e) => CommandResult { ok: false, data: None, error: Some(e) },
        };
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
    use crate::commands::video::get_title_from_url_command;

    // #[test]
    // fn test_download_video_invalido() {
    //     // download_video requer argumentos complexos (Window, Strings, Option)
    //     // Teste comentado até ser adaptado para ambiente de teste adequado
    // }

    #[test]
    fn test_get_title_from_url_invalido() {
        let result = get_title_from_url_command("url_invalida".to_string());
        assert!(!result.ok);
    }
}
use std::io::Write;
use crate::backend::download_manager::{DownloadManager, DownloadTask};
// Integrated pause download (moved from main.rs)
#[tauri::command(rename = "integrated_pause_download")]
pub fn integrated_pause_download(
    state: tauri::State<'_, std::sync::Arc<DownloadManager>>,
    id: String,
    url: String,
    app: tauri::AppHandle,
) {
    let _ = crate::backend::download_progress::get_progress(&url).map(|mut prog| {
        prog.status = "pausado".to_string();
        crate::backend::download_progress::update_progress(&url, prog);
    });
    let mut downloads = state.downloads.lock().unwrap();
    if let Some(task) = downloads.get_mut(&id) {
        task.handle.abort();
    }
    let _ = app.emit("download_paused", serde_json::json!({ "url": url }));
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
    let manager = state.inner().clone();
    let current_size = if std::path::Path::new(&save_path).exists() {
        std::fs::metadata(&save_path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };
    let client = reqwest::Client::new();
    let mut request = client.get(&url);
    if current_size > 0 {
        request = request.header("Range", format!("bytes={}-", current_size));
    }
    let response = request.send().await.map_err(|e| e.to_string())?;
    let mut total_size = response.content_length()
        .or_else(|| {
            response.headers().get("content-range")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split('/').nth(1))
                .and_then(|n| n.parse().ok())
        })
        .unwrap_or(0);
    if total_size == 0 {
        if let Ok(head_resp) = reqwest::Client::new().head(&url).send().await {
            total_size = head_resp.content_length().unwrap_or(0);
        }
    }
    if let Some(parent) = std::path::Path::new(&save_path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&save_path)
        .map_err(|e| {
            println!("[DOWNLOAD][ERRO] Falha ao abrir arquivo: {}", e);
            e.to_string()
        })?;
    let mut stream = response.bytes_stream();
    let mut downloaded = current_size;
    let handle = tokio::spawn({
        let app = app.clone();
        let id = id.clone();
        let username = username.clone();
        let url = url.clone();
        async move {
            use futures_util::StreamExt;
            let mut write_buffer = Vec::with_capacity(1024 * 1024);
            let mut last_emit = std::time::Instant::now();
            while let Some(chunk) = stream.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(_e) => {
                        break;
                    }
                };
                write_buffer.extend_from_slice(&chunk);
                downloaded += chunk.len() as u64;
                if write_buffer.len() >= 1024 * 1024 {
                    if let Err(_e) = file.write_all(&write_buffer) {
                        break;
                    }
                    write_buffer.clear();
                }
                if last_emit.elapsed().as_millis() > 500 {
                    let progress = if total_size > 0 {
                        downloaded as f32 / total_size as f32
                    } else {
                        0.0
                    };
                    let _ = crate::backend::user_service::update_main_url_progress(
                        username.clone(),
                        url.clone(),
                        progress,
                    );
                    let _ = app.emit("download-progress", super::download_manager::ProgressPayload {
                        id: id.clone(),
                        progress: downloaded,
                        total: total_size,
                        status: "downloading".to_string(),
                    });
                    last_emit = std::time::Instant::now();
                }
            }
            if !write_buffer.is_empty() {
                let _ = file.write_all(&write_buffer);
            }
            let progress = if total_size > 0 {
                downloaded as f32 / total_size as f32
            } else {
                1.0
            };
            let _ = crate::backend::user_service::update_main_url_progress(
                username.clone(),
                url.clone(),
                progress,
            );
            let _ = app.emit("download-progress", super::download_manager::ProgressPayload {
                id,
                progress: downloaded,
                total: total_size,
                status: "completed".to_string(),
            });
        }
    });
    let mut downloads = manager.downloads.lock().unwrap();
    downloads.insert(id.clone(), DownloadTask {
        handle,
    });
    Ok(())
}
#[tauri::command]
pub fn pausar_download(username: String, url: String, save_path: String) {
    // println!("[PAUSE COMMAND] INICIADO para URL: {}", url);
    use crate::backend::download_progress::{get_progress, update_progress};
    // println!("[PAUSE COMMAND] Recebido pedido de pausa para URL: {}", url);
    if let Some(mut prog) = get_progress(&url) {
        // println!("[PAUSE COMMAND] Progresso encontrado, status atual: {}", prog.status);
        let downloaded = prog.downloaded as f32;
        let total = prog.total_size as f32;
        let progress = if total > 0.0 { downloaded / total } else { 0.0 };
        let url_for_add = url.clone();
        let url_for_update = url.clone();
        let _url_for_print = url.clone();
        prog.status = "pausado".to_string();
        update_progress(&url, prog);
        let _ = crate::backend::user_service::add_main_url(
            username.clone(),
            url_for_add,
            Some(
                std::path::Path::new(&save_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("video.mp4")
                    .to_string(),
            ),
        );
        let _ = crate::backend::user_service::update_main_url_progress(
            username,
            url_for_update,
            progress,
        );
        // println!("[PAUSE COMMAND] Status atualizado para 'pausado' para URL: {}", _url_for_print);
    } else {
        // println!("[PAUSE COMMAND] Nenhum progresso encontrado para URL: {}", url);
    }
}
use tauri::Emitter;
use crate::backend::downloads::{baixar_video_emit, baixar_player_jmvstream, baixar_hls_emit};
use crate::backend::listing::listar_videos_baixados;
use crate::main_url_title_from_html::get_title_from_url;
use crate::commands::download_manager::CommandResult;
use tauri::Window;
use uuid::Uuid;
use std::collections::HashMap;


#[tauri::command]
pub fn download_video(window: Window, username: String, url: String, filename: String, id: Option<u64>) -> CommandResult<()> {
    let window_clone = window.clone();
    let username_clone = username.clone();
    let url_clone = url.clone();
    let filename_clone = filename.clone();
    std::thread::spawn(move || {
        let result = if url_clone.contains("player.jmvstream.com") {
            baixar_player_jmvstream(&window_clone, &username_clone, &url_clone, &filename_clone, id)
        } else if url_clone.ends_with(".m3u8") || url_clone.contains(".m3u8?") {
            baixar_hls_emit(&window_clone, &username_clone, &url_clone, &filename_clone, id)
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
                window_clone.emit("download_finished", serde_json::json!({ "url": url_clone, "filename": filename_clone, "status": "concluido" }))
            },
            Err(e) => window_clone.emit("download_finished", serde_json::json!({ "url": url_clone, "filename": filename_clone, "status": "erro", "error": e })),
        };
    });
    CommandResult { ok: true, data: Some(()), error: None }
}

#[tauri::command(rename = "download_cascade")]
pub fn download_cascade(_playlist: String, urls: Vec<String>) -> CommandResult<()> {
    // Não há window disponível aqui, então não há emissão de progresso para cascata
    for url in urls {
        let filename = format!("{}.mp4", Uuid::new_v4());
        if let Err(e) = baixar_video_emit(None, &url, &filename) {
            eprintln!("Erro baixar_em_cascata: {}", e);
            return CommandResult { ok: false, data: None, error: Some(e) };
        }
    }
    CommandResult { ok: true, data: Some(()), error: None }
}

#[tauri::command(rename = "list_downloaded_videos")]
pub fn list_downloaded_videos() -> CommandResult<Vec<HashMap<String, String>>> {
    let data = listar_videos_baixados();
    CommandResult { ok: true, data: Some(data), error: None }
}

#[tauri::command(rename = "get_title_from_url")]
pub fn get_title_from_url_command(url: String) -> CommandResult<String> {
    match get_title_from_url(url) {
        Ok(title) => CommandResult { ok: true, data: Some(title), error: None },
        Err(e) => {
            eprintln!("Erro get_title_from_url: {}", e);
            CommandResult { ok: false, data: None, error: Some(e) }
        }
    }
}
