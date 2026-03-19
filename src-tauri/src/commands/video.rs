use crate::backend::downloads::{baixar_video, baixar_hls_emit, baixar_player_jmvstream};
use crate::backend::listing::listar_videos_baixados;
use crate::main_url_title_from_html::get_title_from_url;
use crate::CommandResult;
use tauri::Window;
use uuid::Uuid;
use std::collections::HashMap;

#[tauri::command]
pub fn baixar_video_tauri(window: Window, username: String, url: String, filename: String, id: Option<u64>) -> CommandResult<()> {
    let result = if url.contains("player.jmvstream.com") {
        baixar_player_jmvstream(&window, &username, &url, &filename, id)
    } else if url.ends_with(".m3u8") || url.contains(".m3u8?") {
        baixar_hls_emit(&window, &url, &filename, id)
    } else {
        baixar_video(&url, &filename)
    };
    match result {
        Ok(_) => CommandResult { ok: true, data: Some(()), error: None },
        Err(e) => {
            eprintln!("Erro baixar_video_tauri: {}", e);
            CommandResult { ok: false, data: None, error: Some(e) }
        }
    }
}

#[tauri::command(rename = "baixar_em_cascata")]
pub fn baixar_em_cascata(_playlist: String, urls: Vec<String>) -> CommandResult<()> {
    for url in urls {
        let filename = format!("{}.mp4", Uuid::new_v4());
        if let Err(e) = baixar_video(&url, &filename) {
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
