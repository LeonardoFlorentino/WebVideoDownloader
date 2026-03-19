use crate::backend::playlist_service::{salvar_playlist, marcar_playlist_baixada};
use crate::CommandResult;

#[tauri::command]
pub fn salvar_playlist_tauri(title: String) -> CommandResult<()> {
    match salvar_playlist(title) {
        Ok(_) => CommandResult { ok: true, data: Some(()), error: None },
        Err(e) => {
            eprintln!("Erro salvar_playlist: {}", e);
            CommandResult { ok: false, data: None, error: Some(e) }
        }
    }
}

#[tauri::command]
pub fn marcar_playlist_baixada_tauri(title: String) -> CommandResult<()> {
    match marcar_playlist_baixada(title) {
        Ok(_) => CommandResult { ok: true, data: Some(()), error: None },
        Err(e) => {
            eprintln!("Erro marcar_playlist_baixada: {}", e);
            CommandResult { ok: false, data: None, error: Some(e) }
        }
    }
}
