use crate::backend::playlist_service::{salvar_playlist, marcar_playlist_baixada};
use crate::CommandResult;

#[tauri::command(rename = "save_playlist")]
pub fn save_playlist(title: String) -> CommandResult<()> {
    match salvar_playlist(title) {
        Ok(_) => CommandResult { ok: true, data: Some(()), error: None },
        Err(e) => {
            eprintln!("Erro save_playlist: {}", e);
            CommandResult { ok: false, data: None, error: Some(e) }
        }
    }
}

#[tauri::command(rename = "mark_playlist_downloaded")]
pub fn mark_playlist_downloaded(title: String) -> CommandResult<()> {
    match marcar_playlist_baixada(title) {
        Ok(_) => CommandResult { ok: true, data: Some(()), error: None },
        Err(e) => {
            eprintln!("Erro mark_playlist_downloaded: {}", e);
            CommandResult { ok: false, data: None, error: Some(e) }
        }
    }
}
