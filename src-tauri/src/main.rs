// Comando integrado para pausar download: atualiza status e interrompe a task
#[tauri::command]
fn pause_download_integrado(
    state: tauri::State<'_, Arc<DownloadManager>>,
    id: String,
    url: String,
    app: tauri::AppHandle,
) {
    // Atualiza status para pausado no progresso
    let _ = crate::backend::download_progress::get_progress(&url).map(|mut prog| {
        prog.status = "pausado".to_string();
        crate::backend::download_progress::update_progress(&url, prog);
    });
    // Interrompe a task
    let mut downloads = state.downloads.lock().unwrap();
    if let Some(task) = downloads.get_mut(&id) {
        task.handle.abort();
    }
    // Emite evento para frontend
    let _ = app.emit("download_paused", serde_json::json!({ "url": url }));
}
use tauri::Manager;
use tauri::Emitter;
use std::io::Write;
mod window_utils;
mod commands;
use commands::auth::{cadastrar_usuario_tauri, autenticar_usuario_tauri};
use commands::video::{baixar_video_tauri, baixar_em_cascata, listar_videos_baixados_tauri, get_title_from_url_tauri};
use commands::user::{add_main_url_tauri, get_main_urls_tauri, update_main_url_title_tauri, remove_main_url_tauri};
use commands::playlist::{salvar_playlist_tauri, marcar_playlist_baixada_tauri};
use commands::folder::open_download_folder_tauri;
use serde::Serialize;
use commands::video::pausar_download;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

#[derive(Clone, serde::Serialize)]
struct ProgressPayload {
    id: String,
    progress: u64,
    total: u64,
    status: String, // "downloading" | "paused" | "completed"
}

#[derive(Default)]
struct DownloadManager {
    downloads: Mutex<HashMap<String, DownloadTask>>,
}

struct DownloadTask {
    handle: JoinHandle<()>,
}

// Comando para iniciar ou retomar download
#[tauri::command]
async fn start_download(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<DownloadManager>>,
    id: String,
    url: String,
    save_path: String,
) -> Result<(), String> {
    let manager = state.inner().clone();

    // Verifica tamanho atual do arquivo (resume automático)
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

    // Tenta obter o tamanho total do arquivo
    let mut total_size = response.content_length()
        .or_else(|| {
            response.headers().get("content-range")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split('/').nth(1))
                .and_then(|n| n.parse().ok())
        })
        .unwrap_or(0);

    // Se não conseguiu, tenta um HEAD request
    if total_size == 0 {
        if let Ok(head_resp) = reqwest::Client::new().head(&url).send().await {
            total_size = head_resp.content_length().unwrap_or(0);
        }
    }

    // Garante que a pasta de destino existe
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
        let url_clone = url.clone();
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
                let _ = app.emit("download-progress", ProgressPayload {
                    id: id.clone(),
                    progress: downloaded,
                    total: total_size,
                    status: "downloading".to_string(),
                });
            }
            println!("[DOWNLOAD] Download finalizado para {}. Total baixado: {}", save_path, downloaded);
            // Atualiza status no backend para concluído
            let _ = crate::backend::user_service::update_main_url_status(
                "leona".to_string(), // TODO: pegar username real
                url_clone.clone(),
                "concluído".to_string(),
            );
            let _ = app.emit("download-progress", ProgressPayload {
                id,
                progress: downloaded,
                total: total_size,
                status: "completed".to_string(),
            });
        }
    });

    // Salva a task no manager
    let mut downloads = manager.downloads.lock().unwrap();
    downloads.insert(id.clone(), DownloadTask {
        handle,
    });

    Ok(())
}

// Comando para retomar download (reutiliza start_download)
#[tauri::command]
async fn resume_download(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<DownloadManager>>,
    id: String,
    url: String,
    save_path: String,
) -> Result<(), String> {
    start_download(app, state, id, url, save_path).await
}

// Comando para pausar download
#[tauri::command]
fn pause_download(
    state: tauri::State<'_, Arc<DownloadManager>>,
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

fn main() {
    let manager = Arc::new(DownloadManager::default());

    tauri::Builder::default()
        .manage(manager)
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            tauri::async_runtime::spawn(window_utils::ajustar_janela_metade_tela(window, Some(52.0)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            baixar_video_tauri,
            baixar_em_cascata,
            listar_videos_baixados_tauri,
            get_title_from_url_tauri,
            cadastrar_usuario_tauri,
            autenticar_usuario_tauri,
            add_main_url_tauri,
            get_main_urls_tauri,
            update_main_url_title_tauri,
            salvar_playlist_tauri,
            marcar_playlist_baixada_tauri,
            open_download_folder_tauri,
            remove_main_url_tauri,
            pausar_download,
            start_download,
            pause_download,
            resume_download,
            pause_download_integrado,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
mod main_url_title_from_html;
mod backend {
    pub mod user;
    pub mod user_service;
    pub mod auth;
    pub mod download_progress;
    pub mod filesystem;
    pub mod downloads;
    pub mod playlist;
    pub mod playlist_service;
    pub mod listing;
}

