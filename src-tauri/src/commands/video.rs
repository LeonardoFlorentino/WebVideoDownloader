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
    url: String,
    save_path: String,
) -> Result<(), String> {
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
        println!("[DOWNLOAD] Criando diretório: {:?}", parent);
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    println!("[DOWNLOAD] Abrindo arquivo para escrita: {}", save_path);
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
    println!("[DOWNLOAD] Iniciando download de {} para {} (tamanho total: {})", url, save_path, total_size);
    let handle = tokio::spawn({
        let app = app.clone();
        let id = id.clone();
        async move {
            use futures_util::StreamExt;
            while let Some(chunk) = stream.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => {
                        println!("[DOWNLOAD][ERRO] Falha ao receber chunk: {}", e);
                        break;
                    }
                };
                if let Err(e) = file.write_all(&chunk) {
                    println!("[DOWNLOAD][ERRO] Falha ao escrever chunk: {}", e);
                    break;
                }
                downloaded += chunk.len() as u64;
                println!("[DOWNLOAD] Escreveu {} bytes, total baixado: {}", chunk.len(), downloaded);
                // Atualizar progresso no user.json
                let _ = if total_size > 0 {
                    downloaded as f32 / total_size as f32
                } else {
                    0.0
                };
                // Se desejar persistir progresso, adicione chamada aqui
                let _ = app.emit("download-progress", super::download_manager::ProgressPayload {
                    id: id.clone(),
                    progress: downloaded,
                    total: total_size,
                    status: "downloading".to_string(),
                });
            }
            println!("[DOWNLOAD] Download finalizado para {}. Total baixado: {}", save_path, downloaded);
            // Calcular progresso final (100%)
            let _ = if total_size > 0 {
                downloaded as f32 / total_size as f32
            } else {
                1.0
            };
            // Atualizar status e progresso no user.json
            // Se desejar persistir status/progresso, adicione chamada aqui
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
pub fn pausar_download(url: String) {
    println!("[PAUSE COMMAND] INICIADO para URL: {}", url);
    use crate::backend::download_progress::{get_progress, update_progress};
    println!("[PAUSE COMMAND] Recebido pedido de pausa para URL: {}", url);
    if let Some(mut prog) = get_progress(&url) {
        println!("[PAUSE COMMAND] Progresso encontrado, status atual: {}", prog.status);
        prog.status = "pausado".to_string();
        update_progress(&url, prog);
        println!("[PAUSE COMMAND] Status atualizado para 'pausado' para URL: {}", url);
    } else {
        println!("[PAUSE COMMAND] Nenhum progresso encontrado para URL: {}", url);
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
            baixar_hls_emit(&window_clone, &url_clone, &filename_clone, id)
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
