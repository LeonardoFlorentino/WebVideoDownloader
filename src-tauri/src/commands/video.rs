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
use crate::backend::downloads::{baixar_video_emit, baixar_hls_emit, baixar_player_jmvstream};
use crate::backend::listing::listar_videos_baixados;
use crate::main_url_title_from_html::get_title_from_url;
use crate::CommandResult;
use tauri::Window;
use uuid::Uuid;
use std::collections::HashMap;

#[tauri::command]
pub fn baixar_video_tauri(window: Window, username: String, url: String, filename: String, id: Option<u64>) -> CommandResult<()> {
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
                // Atualiza status no backend para concluído
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

#[tauri::command(rename = "baixar_em_cascata")]
pub fn baixar_em_cascata(_playlist: String, urls: Vec<String>) -> CommandResult<()> {
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

#[tauri::command]
pub fn listar_videos_baixados_tauri() -> CommandResult<Vec<HashMap<String, String>>> {
    let data = listar_videos_baixados();
    CommandResult { ok: true, data: Some(data), error: None }
}

#[tauri::command]
pub fn get_title_from_url_tauri(url: String) -> CommandResult<String> {
    match get_title_from_url(url) {
        Ok(title) => CommandResult { ok: true, data: Some(title), error: None },
        Err(e) => {
            eprintln!("Erro get_title_from_url: {}", e);
            CommandResult { ok: false, data: None, error: Some(e) }
        }
    }
}
